use std::{
    fmt::{self, Display, Formatter},
    mem::transmute,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    Return = 0,
    LoadConst = 1,
    LoadConstLong = 2,

    Add = 3,
    Sub = 4,
    Mul = 5,
    Div = 6,
    Mod = 7,

    Min = 8,
    Max = 9,

    And = 10,
    Or = 11,
    Xor = 12,

    Not = 13,

    Concat = 14,

    /// Print
    Prt = 15,
    /// Print with newline
    PrtL = 16,

    PopN = 21,
    PopNLong = 22,

    WriteSt = 23,
    WriteStLong = 24,

    ReadSt = 25,
    ReadStLong = 26,

    ReadLine = 27,
    ReadLineLong = 28,

    /// FN, arity (max 256)
    FnDef = 29,

    JmpFalse = 30,
    JmpFalseIt = 40,
    Jmp = 31,

    WriteIt = 32,
    ReadIt = 33,

    Equals = 34,

    Call = 35,
    InterpStr = 36,

    Cast = 37,
    CastMut = 38,
    CastMutLong = 39,

    GT = 41,
    LT = 42,
    LTE = 43,
    GTE = 44,
}

use OpCode::*;

impl OpCode {
    pub fn arity(self) -> usize {
        match self {
            // 8-bit index
            CastMut | Cast | FnDef | Call | ReadLine | ReadSt | WriteSt | PopN | LoadConst => 1,
            // 24-bit index
            CastMutLong | ReadLineLong | ReadStLong | WriteStLong | PopNLong | LoadConstLong => 3,

            JmpFalseIt | JmpFalse | Jmp => 4,

            GT | LT | LTE | GTE | InterpStr | Equals | ReadIt | WriteIt | Return | Prt | PrtL
            | Not | Xor | Or | And | Add | Mul | Div | Sub | Mod | Min | Max | Concat => 0,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Return => "ret",
                LoadConst => "ldc",
                LoadConstLong => "ldcl",

                Add => "add",
                Sub => "sub",
                Mul => "mul",
                Div => "div",
                Mod => "mod",

                Min => "min",
                Max => "max",

                Xor => "xor",
                And => "and",
                Or => "or",

                LTE => "lte",
                GTE => "gte",
                LT => "lt",
                GT => "gt",

                Equals => "eq",

                Not => "not",
                Concat => "cnct",

                Prt => "prt",
                PrtL => "prtl",

                PopN => "pop",
                PopNLong => "popl",

                WriteSt => "wrt",
                WriteStLong => "wrtl",

                ReadSt => "rd",
                ReadStLong => "rdl",

                ReadLine => "rln",
                ReadLineLong => "rlnl",

                FnDef => "fndf",
                Call => "call",

                JmpFalse => "jmpf",
                JmpFalseIt => "jpfi",
                Jmp => "jmp",

                ReadIt => "rit",
                WriteIt => "wit",

                InterpStr => "ints",
                Cast => "cst",
                CastMut => "ctm",
                CastMutLong => "ctml",
            }
        )
    }
}

#[inline(always)]
pub unsafe fn byte_to_opcode(instr: u8) -> OpCode {
    // SAFTEY: the opcode we are transmuting into is literally
    // a u8, and is represented as a u8, so this is sound.
    transmute(instr)
}
