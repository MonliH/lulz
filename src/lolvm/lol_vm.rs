use smallvec::{smallvec, SmallVec};
use std::{
    io::{self, BufRead},
    mem,
    rc::Rc,
};

use super::{CallFrame, Stack};
use crate::{
    diagnostics::prelude::*,
    frontend::ast::Type,
    lolbc::{
        bits::usize_from_u8,
        byte_to_opcode, disasm_instruction, Chunk,
        OpCode::*,
        Value::{self, *},
    },
};

pub struct LolVm {
    st: Stack,
    call_st: SmallVec<[CallFrame; 1024]>,
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
    ($stack: expr, $floatf: expr, $intf: expr, $span: expr, $name: expr) => {
        binary_num!($stack, $floatf, $intf, $span, $name, Float, Int)
    };
    ($stack: expr, $floatf: expr, $intf: expr, $span: expr, $name: expr, $floatout: ident, $intout: ident) => {{
        let snd_st = $stack.pop();
        let snd = snd_st.clone().cast_try_num().ok_or_else(|| {
            Diagnostics::from(
                Diagnostic::build(Level::Error, DiagnosticType::Runtime, $span).annotation(
                    Cow::Owned(format!(
                        "could not convert second operand {} to a NUMBR or NUMBAR",
                        snd_st.ty()
                    )),
                    $span,
                ),
            )
        })?;

        let fst_st = $stack.pop();
        let fst = fst_st.clone().cast_try_num().ok_or_else(|| {
            Diagnostics::from(
                Diagnostic::build(Level::Error, DiagnosticType::Runtime, $span).annotation(
                    Cow::Owned(format!(
                        "could not convert second operand {} to a NUMBR or NUMBAR",
                        snd_st.ty()
                    )),
                    $span,
                ),
            )
        })?;

        let new_val = match (fst, snd) {
            (Int(i1), Int(i2)) => $intout($intf(i1, i2)),
            (Float(f1), Float(f2)) => $floatout($floatf(f1, f2)),
            (Int(i), Float(f)) => $floatout($floatf(i as f64, f)),
            (Float(f), Int(i)) => $floatout($floatf(f, i as f64)),
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
    #[inline(always)]
    fn frame_mut(&mut self) -> &mut CallFrame {
        let len = self.call_st.len();
        &mut self.call_st[len - 1]
    }

    #[inline(always)]
    fn frame(&self) -> &CallFrame {
        &self.call_st[self.call_st.len() - 1]
    }

    #[inline(always)]
    fn read_8b(&mut self) -> u8 {
        let b = self.c.bytecode[self.frame().ip];
        self.frame_mut().ip += 1;
        b
    }

    #[inline(always)]
    fn read_32b(&mut self) -> usize {
        let hi = self.read_8b();
        let mih = self.read_8b();
        let mil = self.read_8b();
        let lo = self.read_8b();
        u32::from_le_bytes([hi, mih, mil, lo]) as usize
    }

    #[inline(always)]
    fn read_24b(&mut self) -> usize {
        let hi = self.read_8b();
        let mi = self.read_8b();
        let lo = self.read_8b();
        usize_from_u8(hi, mi, lo)
    }

    #[inline(always)]
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
            let op = unsafe { byte_to_opcode(self.read_8b()) };
            match op {
                Return => {
                    let res = self.st.pop();
                    self.call_st.pop();
                    if self.call_st.is_empty() {
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

                GTE => binary_num!(
                    self.st,
                    (|a, b| a >= b),
                    (|a, b| a >= b),
                    self.c.pos.get(op_loc),
                    "compare with SAEM + BIGGR to",
                    Bool,
                    Bool
                ),

                LTE => binary_num!(
                    self.st,
                    (|a, b| a <= b),
                    (|a, b| a <= b),
                    self.c.pos.get(op_loc),
                    "compare with SAEM + SMALLR to",
                    Bool,
                    Bool
                ),

                LT => binary_num!(
                    self.st,
                    (|a, b| a < b),
                    (|a, b| a < b),
                    self.c.pos.get(op_loc),
                    "compare with DIFFRINT + BIGGR to",
                    Bool,
                    Bool
                ),

                GT => binary_num!(
                    self.st,
                    (|a, b| a > b),
                    (|a, b| a > b),
                    self.c.pos.get(op_loc),
                    "compare with DIFFRINT + BIGGR to",
                    Bool,
                    Bool
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
                    let st2 = self.st.pop();
                    let st1 = self.st.pop();
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

                JmpFalseIt => {
                    let cond = self.it.to_bool();
                    let offset = self.read_32b();
                    if !cond {
                        self.frame_mut().ip += offset;
                    }
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

                InterpStr => {
                    let inp_str = self.st.pop();
                    if let IStr(s, stpos) = inp_str {
                        let mut remaining_s = &s[..];
                        let mut st_pieces = Vec::with_capacity(stpos.len() * 2 + 1);
                        for (idx, st) in stpos {
                            let (fst, snd) = remaining_s.split_at(idx);
                            st_pieces.push(Cow::Borrowed(fst));
                            let val = self.st[st].to_str().to_string();
                            st_pieces.push(Cow::Owned(val));
                            remaining_s = snd;
                        }
                        st_pieces.push(Cow::Borrowed(remaining_s));
                        self.st.push(Str(Rc::from(st_pieces.concat())));
                    } else {
                        unreachable!()
                    }
                }

                Cast => {
                    let ty = Type::from_num(self.read_8b()).unwrap();
                    let e = self.st.pop().cast(ty, self.c.pos.get(op_loc))?;
                    self.st.push(e);
                }

                CastMut | CastMutLong => {
                    let st_pos = match op {
                        CastMut => self.read_8b() as usize,
                        CastMutLong => self.read_24b(),
                        _ => unreachable!(),
                    } + f.st_offset;
                    let ty = Type::from_num(self.read_8b()).unwrap();
                    let e = mem::take(&mut self.st[st_pos]).cast(ty, self.c.pos.get(op_loc))?;
                    self.st[st_pos] = e;
                }
            }
        }
    }

    #[inline]
    fn call(&mut self, mem_pos: usize, args: usize) {
        self.call_st.push(CallFrame {
            ret_ip: self.frame().ip,
            ip: mem_pos,
            st_offset: self.st.len() - 1 - args,
        });
    }

    #[inline]
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
            self.call(mem_pos, args as usize);
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
