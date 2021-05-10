//! LOLVM bytecode
pub mod bits;
mod chunk;
mod disasm;
mod opcodes;
mod values;
pub use chunk::{ByteC, Chunk, LSpan, Positions};
pub use disasm::{disasm, disasm_instruction};
pub use opcodes::{byte_to_opcode, OpCode, NUM_CODES};
pub use values::{Value, ValueArray};
