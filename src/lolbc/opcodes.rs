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

    Asgn = 17,
    AsgnLong = 19,
    VDec = 18,
    VDecLong = 20,

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
    Jmp = 31,

    WriteIt = 32,
    ReadIt = 33,
}

use OpCode::*;

pub const NUM_CODES: u8 = ReadIt as u8;

impl OpCode {
    pub fn arity(self) -> usize {
        match self {
            // 8-bit index
            ReadLine | ReadSt | WriteSt | PopN | Asgn | VDec | LoadConst => 1,
            // 24-bit index
            ReadLineLong | ReadStLong | WriteStLong | PopNLong | AsgnLong | VDecLong
            | LoadConstLong => 3,

            JmpFalse | Jmp => 4,

            FnDef => 1,

            ReadIt | WriteIt | Return | Prt | PrtL | Not | Xor | Or | And | Add | Mul | Div
            | Sub | Mod | Min | Max | Concat => 0,
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

                Not => "not",
                Concat => "cnct",

                Prt => "prt",
                PrtL => "prtl",
                Asgn => "asn",
                VDec => "dec",

                AsgnLong => "asnl",
                VDecLong => "decl",

                PopN => "pop",
                PopNLong => "popl",

                WriteSt => "wrt",
                WriteStLong => "wrtl",

                ReadSt => "rd",
                ReadStLong => "rdl",

                ReadLine => "rln",
                ReadLineLong => "rlnl",

                FnDef => "fnd",

                JmpFalse => "jmpf",
                Jmp => "jmp",

                ReadIt => "rit",
                WriteIt => "wit",
            }
        )
    }
}

pub fn byte_to_opcode(instr: u8) -> Option<OpCode> {
    if instr > NUM_CODES {
        None
    } else {
        // SAFTEY: the opcode we are transmuting into is literally
        // a u8, and is represented as a u8, so this is sound.
        //
        // We also do bounds checking above, so if it's an invalid
        // u8 it's also sound.
        Some(unsafe { transmute(instr) })
    }
}
