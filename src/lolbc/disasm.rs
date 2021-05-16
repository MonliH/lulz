use crate::lolbc::opcodes::byte_to_opcode;

use super::bits;
use super::{Chunk, OpCode};

pub fn disasm(chunk: &Chunk) {
    eprintln!("=== Disassembled Bytecode ===");
    let bytecode = &chunk.bytecode;

    let mut offset = 0;
    while offset < bytecode.len() {
        offset = disasm_instruction(&chunk, offset);
    }
    eprintln!("===\n");
}

pub fn disasm_instruction(chunk: &Chunk, offset: usize) -> usize {
    let bytecode = &chunk.bytecode;
    let instr = byte_to_opcode(bytecode[offset]);
    let pos = chunk.pos.get(offset);
    eprint!("{:0>5}  {:>4}:{: <3} ", offset, pos.s, pos.e);
    match instr {
        Some(op) => {
            eprint!("{:>4} ", op);
            let arity = op.arity();
            for operands in (offset + 1)..(offset + 1 + arity) {
                eprint!("{} ", bytecode[operands]);
            }

            // Special cases
            match op {
                OpCode::LoadConst => eprint!(
                    "(value: {})",
                    chunk.values.load(bytecode[offset + 1] as usize),
                ),
                OpCode::LoadConstLong => eprint!(
                    "(value: {})",
                    chunk.values.load(bits::usize_from_u8(
                        bytecode[offset + 1],
                        bytecode[offset + 2],
                        bytecode[offset + 3]
                    )),
                ),
                _ => {}
            }

            eprint!("\n");
            offset + 1 + arity
        }
        None => {
            eprintln!("invalid ({})", bytecode[offset]);
            offset + 1
        }
    }
}
