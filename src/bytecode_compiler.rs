use crate::{
    frontend::ast::Block,
    lolbc::{Chunk, OpCode},
};

pub struct BytecodeCompiler {}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, ast: Block) -> Chunk {
        let mut chunk = Chunk::new("TEST CHUNK".to_string());

        chunk.write_instr(OpCode::Return as u8, (1, 1));
        chunk.write_get_const(123.0, (2, 1));
        chunk.write_get_const(256.0, (3, 1));
        chunk.write_get_const(123021.0, (4, 1));

        chunk
    }
}
