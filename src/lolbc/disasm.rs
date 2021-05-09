use std::intrinsics::transmute;

use super::bits;
use super::{Chunk, OpCode, NUM_CODES};

pub fn disasm(chunk: &Chunk) {
    eprintln!("=== {} ===", chunk.name);
    let bytecode = &chunk.bytecode;

    let mut offset = 0;
    while offset < bytecode.len() {
        offset = disasm_instruction(&chunk, offset);
    }
}

fn disasm_instruction(chunk: &Chunk, offset: usize) -> usize {
    let bytecode = &chunk.bytecode;
    let instr = getop(bytecode[offset]);
    let pos = chunk.pos.get(offset);
    print!("{:0>5}  {:>4}:{: <3} ", offset, pos.0, pos.1);
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

fn getop(instr: u8) -> Option<OpCode> {
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
