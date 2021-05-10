use super::Stack;
use crate::{
    diagnostics::prelude::*,
    lolbc::{bits::usize_from_u8, byte_to_opcode, disasm_instruction, Chunk, OpCode::*, Value::*},
};

const DEBUG: bool = true;

#[derive(Default)]
pub struct LolVm {
    ip: usize,
    c: Chunk,
    st: Stack,
}

macro_rules! binary_bool {
    ($stack: expr, $boolf: expr) => {{
        let fst = $stack.pop().to_bool();
        let snd = $stack.pop().to_bool();

        $stack.push(Bool($boolf(fst, snd)));

        panic!("Internal error: `{}` and `{}` are not bools", fst, snd);
    }};
}

macro_rules! binary_num {
    ($stack: expr, $floatf: expr, $intf: expr, $span: expr, $name: expr) => {{
        let fst = $stack.pop().cast_try_num($span, "first operand")?;
        let snd = $stack.pop().cast_try_num($span, "first operand")?;

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
    pub fn byte(&mut self) -> u8 {
        let b = self.c.bytecode[self.ip];
        self.ip += 1;
        b
    }

    pub fn run(&mut self, chunk: Chunk) -> Failible<()> {
        self.ip = 0;
        self.c = chunk;
        println!("{} {}", self.c.bytecode.len(), self.c.pos.len());
        loop {
            if DEBUG {
                print!("                ");
                for value in &self.st {
                    print!("[ {} ]", value);
                }
                print!("\n");
                disasm_instruction(&self.c, self.ip);
            }

            let op_loc = self.ip;
            let op = byte_to_opcode(self.byte()).expect("Internal error: unknown opcode");
            match op {
                Return => {
                    println!("{}", self.st.pop());
                    return Ok(());
                }
                LoadConst => {
                    let loc = self.byte();
                    self.st.push(self.c.values.load(loc as usize));
                }
                LoadConstLong => {
                    let hi = self.byte();
                    let mi = self.byte();
                    let lo = self.byte();
                    self.st.push(self.c.values.load(usize_from_u8(hi, mi, lo)));
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
                    (|a, b| a - b),
                    (|a, b| a - b),
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
                    let st1 = self.st.pop().to_str();
                    let st2 = self.st.pop().to_str();

                    self.st.push(Str(format!("{}{}", st1, st2).into()));
                }

                And => binary_bool!(self.st, (|a, b| a && b)),
                Or => binary_bool!(self.st, (|a, b| a || b)),
                Xor => binary_bool!(self.st, (|a, b| a ^ b)),
            }
        }
    }
}
