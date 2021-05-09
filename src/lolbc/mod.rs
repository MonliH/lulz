//! LOLVM bytecode
mod bits;
mod chunk;
mod disasm;
mod opcodes;
mod values;
pub use chunk::{ByteC, ByteCRef, Chunk, LSpan, Positions};
pub use disasm::disasm;
pub use opcodes::{OpCode, NUM_CODES};
pub use values::{Value, ValueArray};
