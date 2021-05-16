use smallvec::{smallvec, SmallVec};
use std::io;
use std::io::BufRead;

use super::{CallFrame, Stack};
use crate::{
    diagnostics::prelude::*,
    lolbc::{
        bits::usize_from_u8,
        byte_to_opcode, disasm_instruction, Chunk,
        OpCode::*,
        Value::{self, *},
    },
};

pub struct LolVm {
    st: Stack,
    call_st: SmallVec<[CallFrame; 256]>,
    it: Value,
    c: Chunk,
}

macro_rules! binary_bool {
    ($stack: expr, $boolf: expr) => {{
        let snd = $stack.pop().to_bool();
        let fst = $stack.pop().to_bool();

        $stack.push(Bool($boolf(fst, snd)));
    }};
}

macro_rules! binary_num {
    ($stack: expr, $floatf: expr, $intf: expr, $span: expr, $name: expr) => {{
        let snd = $stack.pop().cast_try_num($span, "second operand")?;
        let fst = $stack.pop().cast_try_num($span, "first operand")?;

        let new_val = match (fst, snd) {
            (Float(f1), Float(f2)) => Float($floatf(f1, f2)),
            (Int(i), Float(f)) => Float($floatf(i as f64, f)),
            (Float(f), Int(i)) => Float($floatf(f, i as f64)),
            (Int(i1), Int(i2)) => Int($intf(i1, i2)),
            (fst, snd) => {
                return Err(Diagnostic::build(Level::Error, DiagnosticType::Type, $span)
                    .annotation(
                        Cow::Owned(format!(
                            "cannot {} types `{}` and `{}`",
                            $name,
                            fst.ty(),
                            snd.ty()
                        )),
                        $span,
                    )
                    .into())
            }
        };

        $stack.push(new_val);
    }};
}

impl LolVm {
    #[inline]
    fn frame_mut(&mut self) -> &mut CallFrame {
        let len = self.call_st.len();
        &mut self.call_st[len - 1]
    }

    #[inline]
    fn frame(&self) -> &CallFrame {
        &self.call_st[self.call_st.len() - 1]
    }

    #[inline]
    fn read_8b(&mut self) -> u8 {
        let b = self.c.bytecode[self.frame().ip];
        self.frame_mut().ip += 1;
        b
    }

    #[inline]
    fn read_32b(&mut self) -> usize {
        let hi = self.read_8b();
        let mih = self.read_8b();
        let mil = self.read_8b();
        let lo = self.read_8b();
        u32::from_le_bytes([hi, mih, mil, lo]) as usize
    }

    #[inline]
    fn read_24b(&mut self) -> usize {
        let hi = self.read_8b();
        let mi = self.read_8b();
        let lo = self.read_8b();
        usize_from_u8(hi, mi, lo)
    }

    fn peek_st(&mut self, idx: usize) -> Value {
        self.st[self.st.len() - idx - 1].clone()
    }

    pub fn new() -> Self {
        Self {
            st: Stack::new(),
            it: Null,
            call_st: smallvec![CallFrame {
                ret_ip: 0,
                ip: 0,
                st_offset: 0
            }],
            c: Chunk::default(),
        }
    }

    pub fn run(&mut self, chunk: Chunk, debug: bool) -> Failible<()> {
        self.c = chunk;
        let mut f = *self.frame();
        loop {
            if debug {
                eprint!("                ");
                for value in &self.st {
                    eprint!("[ {} ]", value);
                }
                eprint!("\n");
                disasm_instruction(&self.c, self.frame().ip);
            }

            let op_loc = self.frame().ip;
            let op = byte_to_opcode(self.read_8b()).expect("Internal error: unknown opcode");
            match op {
                Return => {
                    let res = self.st.pop();
                    self.call_st.pop();
                    if self.call_st.is_empty() {
                        self.st.pop();
                        return Ok(());
                    }

                    let no = self.st.len() - f.st_offset;
                    self.st.popn(no);
                    f = self.call_st[self.call_st.len() - 1];
                    self.st.push(res);
                }

                LoadConst => {
                    let loc = self.read_8b();
                    self.st.push(self.c.values.load(loc as usize));
                }

                LoadConstLong => {
                    let addr = self.read_24b();
                    self.st.push(self.c.values.load(addr));
                }

                Add => binary_num!(
                    self.st,
                    (|a, b| a + b),
                    (|a, b| a + b),
                    self.c.pos.get(op_loc),
                    "get SUM of"
                ),

                Sub => binary_num!(
                    self.st,
                    (|a, b| a - b),
                    (|a, b| a - b),
                    self.c.pos.get(op_loc),
                    "get DIFF of"
                ),

                Div => binary_num!(
                    self.st,
                    (|a, b| a / b),
                    (|a, b| a / b),
                    self.c.pos.get(op_loc),
                    "get QUOSHUNT of"
                ),

                Mul => binary_num!(
                    self.st,
                    (|a, b| a * b),
                    (|a, b| a * b),
                    self.c.pos.get(op_loc),
                    "get PRODUKT of"
                ),

                Mod => binary_num!(
                    self.st,
                    (|a, b| a % b),
                    (|a, b| a % b),
                    self.c.pos.get(op_loc),
                    "get MOD of"
                ),

                Min => binary_num!(
                    self.st,
                    (|a, b| f64::min(a, b)),
                    (|a, b| i64::min(a, b)),
                    self.c.pos.get(op_loc),
                    "get SMALLR of"
                ),

                Max => binary_num!(
                    self.st,
                    (|a, b| f64::max(a, b)),
                    (|a, b| i64::max(a, b)),
                    self.c.pos.get(op_loc),
                    "get BIGGR of"
                ),

                Not => {
                    let b = self.st.pop().to_bool();
                    self.st.push(Bool(!b));
                }

                Concat => {
                    let st2 = self.st.pop().to_str();
                    let st1 = self.st.pop().to_str();

                    self.st.push(Str(format!("{}{}", st1, st2).into()));
                }

                And => binary_bool!(self.st, (|a, b| a && b)),
                Or => binary_bool!(self.st, (|a, b| a || b)),
                Xor => binary_bool!(self.st, (|a, b| a ^ b)),

                Prt => {
                    print!("{}", self.st.pop().disp());
                }
                PrtL => {
                    println!("{}", self.st.pop().disp());
                }

                Equals => {
                    let st2 = self.st.pop().to_str();
                    let st1 = self.st.pop().to_str();
                    self.st.push(Bool(st1 == st2));
                }

                PopN => {
                    let no = self.read_8b();
                    self.st.popn(no as usize);
                }

                PopNLong => {
                    let no = self.read_24b();
                    self.st.popn(no);
                }

                WriteSt => {
                    let stid = self.read_8b();
                    self.st[f.st_offset + stid as usize] = self.st.pop();
                }

                WriteStLong => {
                    let stid = self.read_24b();
                    self.st[f.st_offset + stid] = self.st.pop();
                }

                ReadSt => {
                    let stid = self.read_8b();
                    self.st.push(self.st[f.st_offset + stid as usize].clone());
                }

                ReadStLong => {
                    let stid = self.read_24b();
                    self.st.push(self.st[f.st_offset + stid].clone());
                }

                ReadLine => {
                    let dest = self.read_8b();
                    let line = self.read_stdin(op_loc)?;
                    self.st[f.st_offset + dest as usize] = Str(line.into());
                }

                ReadLineLong => {
                    let dest = self.read_24b();
                    let line = self.read_stdin(op_loc)?;
                    self.st[f.st_offset + dest] = Str(line.into());
                }

                FnDef => {
                    self.read_8b();
                }

                Call => {
                    let arg_count = self.read_8b();
                    let fun = self.peek_st(arg_count as usize);
                    self.call_value(fun, arg_count, f.ip + op_loc)?;
                    f = *self.frame();
                }

                Jmp => {
                    let offset = self.read_32b();
                    self.frame_mut().ip += offset;
                }

                JmpFalse => {
                    let offset = self.read_32b();
                    let cond = self.st.pop().to_bool();
                    if !cond {
                        self.frame_mut().ip += offset;
                    }
                }

                WriteIt => {
                    self.it = self.st.pop();
                }

                ReadIt => {
                    self.st.push(self.it.clone());
                }
            }
        }
    }

    fn call(&mut self, mem_pos: usize, args: usize) -> Failible<()> {
        self.call_st.push(CallFrame {
            ret_ip: self.frame().ip,
            ip: mem_pos,
            st_offset: self.st.len() - 1 - args,
        });
        Ok(())
    }

    fn call_value(&mut self, fun: Value, args: u8, call_instr: usize) -> Failible<()> {
        if let Fun(mem_pos) = fun {
            let func_args = self.c.bytecode[mem_pos + 1];
            if func_args != args {
                let call_span = self.c.pos.get(call_instr);
                return Err(Diagnostic::build(
                    Level::Error,
                    DiagnosticType::FunctionArgumentMismatch,
                    call_span,
                )
                .annotation(
                    Cow::Owned(format!(
                        "this funkshon takes {} {}...",
                        func_args,
                        plural(func_args as usize, "argument")
                    )),
                    self.c.pos.get(mem_pos),
                )
                .annotation(
                    Cow::Owned(format!(
                        "but {} {} are passed in here",
                        args,
                        plural(args as usize, "argument")
                    )),
                    call_span,
                )
                .into());
            }
            self.call(mem_pos, args as usize)?;
            Ok(())
        } else {
            let span = self.c.pos.get(call_instr);
            Err(Diagnostic::build(Level::Error, DiagnosticType::Type, span)
                .annotation(
                    Cow::Owned(format!("value is a `{}` not a `FUNKSHON`", fun.ty())),
                    span,
                )
                .into())
        }
    }

    fn read_stdin(&self, op_loc: usize) -> Failible<String> {
        let stdin = io::stdin();
        let mut stdin_iter = stdin.lock().lines();
        stdin_iter.next().unwrap().map_err(|e| {
            Diagnostics::from(
                Diagnostic::build(
                    Level::Error,
                    DiagnosticType::Runtime,
                    self.c.pos.get(op_loc),
                )
                .annotation(
                    Cow::Owned(format!("failed to read from stdin: `{}`", e)),
                    self.c.pos.get(op_loc),
                ),
            )
        })
    }
}
