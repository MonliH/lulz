use crate::{
    diagnostics::{Failible, Span},
    frontend::ast::{Block, Expr},
    lolbc::{Chunk, OpCode, Value},
};

pub struct BytecodeCompiler {
    c: Chunk,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            c: Chunk::new("default".to_string()),
        }
    }

    pub fn compile_expr(&mut self, expr: Expr) -> Failible<()> {
        Ok(())
    }

    pub fn compile(&mut self, ast: Block) -> Failible<Chunk> {
        self.c.write_instr(OpCode::Return as u8, Span::default());
        self.c.write_get_const(Value::Float(123.0), Span::default());
        self.c.write_get_const(Value::Bool(true), Span::default());
        self.c.write_get_const(Value::Int(19320), Span::default());
        self.c.write_get_const(Value::Null, Span::default());

        Ok(std::mem::replace(
            &mut self.c,
            Chunk::new("default".to_string()),
        ))
    }
}
