//! LOLVM bytecode
mod bits;
mod chunk;
mod disasm;
mod opcodes;
mod values;
pub use chunk::{Chunk, LSpan, Positions};
pub use disasm::disasm;
pub use opcodes::{OpCode, NUM_CODES};
pub use values::{Value, ValueArray};

pub type ByteC = Vec<u8>;
pub type ByteCRef<'a> = &'a [u8];
