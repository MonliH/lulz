use crate::lolbc::opcodes::byte_to_opcode;

use super::bits;
use super::{Chunk, OpCode};

pub fn disasm(chunk: &Chunk) {
    eprintln!("=== {} ===", chunk.name);
    let bytecode = &chunk.bytecode;

    let mut offset = 0;
    while offset < bytecode.len() {
        offset = disasm_instruction(&chunk, offset);
    }
}

pub fn disasm_instruction(chunk: &Chunk, offset: usize) -> usize {
    let bytecode = &chunk.bytecode;
    let instr = byte_to_opcode(bytecode[offset]);
    let pos = chunk.pos.get(offset);
    print!("{:0>5}  {:>4}:{: <3} ", offset, pos.s, pos.e);
    match instr {
        Some(op) => {
            print!("{:>4} ", op);
            let arity = op.arity();
            for operands in (offset + 1)..(offset + 1 + arity) {
                print!("{} ", bytecode[operands]);
            }

            // Special cases
            match op {
                OpCode::LoadConst => print!(
                    "(value: {})",
                    chunk.values.load(bytecode[offset + 1] as usize),
                ),
                OpCode::LoadConstLong => print!(
                    "(value: {})",
                    chunk.values.load(bits::usize_from_u8(
                        bytecode[offset + 1],
                        bytecode[offset + 2],
                        bytecode[offset + 3]
                    )),
                ),
                _ => {}
            }

            print!("\n");
            offset + 1 + arity
        }
        None => {
            println!("invalid ({})", bytecode[offset]);
            offset + 1
        }
    }
}
