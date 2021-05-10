use crate::{
    diagnostics::{Failible, Span},
    frontend::ast::{Block, Expr, ExprKind, OpTy, StatementKind},
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
        match expr.expr_kind {
            ExprKind::Float(f) => {
                self.c.write_get_const(Value::Float(f), expr.span);
            }

            ExprKind::Int(i) => {
                self.c.write_get_const(Value::Int(i), expr.span);
            }

            ExprKind::Bool(b) => {
                self.c.write_get_const(Value::Bool(b), expr.span);
            }

            ExprKind::String(s) => {
                self.c.write_get_const(Value::Str(s.into()), expr.span);
            }

            ExprKind::Not(e) => {
                self.compile_expr(*e)?;
                self.write_instr(OpCode::Not, expr.span)
            }

            ExprKind::Operator(op, e1, e2) => {
                self.compile_expr(*e1)?;
                self.compile_expr(*e2)?;
                self.write_instr(
                    match op {
                        OpTy::Add => OpCode::Add,
                        OpTy::Sub => OpCode::Sub,
                        OpTy::Mul => OpCode::Mul,
                        OpTy::Div => OpCode::Div,
                        OpTy::Mod => OpCode::Mod,

                        OpTy::Min => OpCode::Min,
                        OpTy::Max => OpCode::Max,

                        _ => todo!(),
                    },
                    expr.span,
                );
            }
            _ => {}
        }
        Ok(())
    }

    pub fn write_instr(&mut self, opcode: OpCode, span: Span) {
        self.c.write_instr(opcode as u8, span);
    }

    pub fn compile(&mut self, ast: Block) -> Failible<Chunk> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Expr(e) => {
                    self.compile_expr(e)?;
                }
                _ => {}
            }
        }

        self.write_instr(OpCode::Return, ast.1);

        Ok(std::mem::replace(
            &mut self.c,
            Chunk::new("default".to_string()),
        ))
    }
}
