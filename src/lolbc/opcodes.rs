use std::fmt::{self, Display, Formatter};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OpCode {
    Return = 0,
    LoadConst = 1,
    LoadConstLong = 2,
}

pub const NUM_CODES: u8 = OpCode::LoadConstLong as u8;

impl OpCode {
    pub fn arity(self) -> usize {
        match self {
            Self::Return => 0,
            // 8-bit index
            Self::LoadConst => 1,
            // 24-bit index
            Self::LoadConstLong => 3,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Return => "ret",
                Self::LoadConst => "ldc",
                Self::LoadConstLong => "ldcl",
            }
        )
    }
}
