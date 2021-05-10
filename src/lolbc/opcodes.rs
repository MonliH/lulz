use std::{
    fmt::{self, Display, Formatter},
    mem::transmute,
};

#[repr(u8)]
#[derive(Clone, Copy)]
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
}

use OpCode::*;

pub const NUM_CODES: u8 = Not as u8;

impl OpCode {
    pub fn arity(self) -> usize {
        match self {
            Return => 0,
            // 8-bit index
            LoadConst => 1,
            // 24-bit index
            LoadConstLong => 3,

            Not | Xor | Or | And | Add | Mul | Div | Sub | Mod | Min | Max => 0,
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
