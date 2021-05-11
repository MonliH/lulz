use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, Ident, OpTy, StatementKind},
    lolbc::{bits::Bits, Chunk, OpCode, Value},
    lolvm::StrId,
};

pub struct Local {
    depth: usize,
    name: StrId,
}

pub struct BytecodeCompiler {
    c: Chunk,
    locals: SmallVec<[Local; 16]>,
    // map from interned string to a local usize
    valid_locals: FxHashMap<StrId, usize>,
    scope_depth: usize,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            c: Chunk::new("default".to_string()),
            locals: SmallVec::new(),
            valid_locals: FxHashMap::default(),
            scope_depth: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn write_bits(&mut self, bits: Bits, short: OpCode, long: OpCode) {
        match bits {
            Bits::U8(addr) => {
                self.write_instr(short, Span::default());
                self.c.write_instr(addr, Span::default());
            }

            Bits::U24(hi, mi, lo) => {
                self.write_instr(long, Span::default());
                self.c.write_instr(hi, Span::default());
                self.c.write_instr(mi, Span::default());
                self.c.write_instr(lo, Span::default());
            }
        }
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        let mut pops = 0;
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].depth > self.scope_depth
        {
            let str_id = self.locals.last().unwrap().name;
            self.valid_locals.remove(&str_id);
            pops += 1;
        }
        self.locals.truncate(self.locals.len() - pops);
        let bits = Bits::from(pops);
        self.write_bits(bits, OpCode::PopN, OpCode::PopNLong);
    }

    fn compile_seq(&mut self, es: Vec<Expr>, op: OpCode) -> Failible<()> {
        let len = es.len();
        let mut es_iter = es.into_iter();
        if len == 1 {
            self.compile_expr(es_iter.next().unwrap())?;
        } else {
            let first = es_iter.next().unwrap();
            let second = es_iter.next().unwrap();
            let mut combined_span = first.span.combine(&second.span);
            self.compile_expr(first)?;
            self.compile_expr(second)?;
            self.write_instr(op, combined_span);
            for es in es_iter {
                combined_span = combined_span.combine(&es.span);
                self.compile_expr(es)?;
                self.write_instr(op, combined_span);
            }
        }
        Ok(())
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

            ExprKind::Null => {
                self.c.write_get_const(Value::Null, expr.span);
            }

            ExprKind::Not(e) => {
                self.compile_expr(*e)?;
                self.write_instr(OpCode::Not, expr.span);
            }

            ExprKind::Variable(id) => {
                let interned = self.c.interner.intern(id.0.as_str());
                let stid = self.resolve_local(interned, id.1)?;
                self.write_bits(stid.into(), OpCode::ReadSt, OpCode::ReadStLong);
            }

            ExprKind::All(es) => self.compile_seq(es, OpCode::And)?,
            ExprKind::Any(es) => self.compile_seq(es, OpCode::Or)?,
            ExprKind::Concat(es) => self.compile_seq(es, OpCode::Concat)?,

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

                        OpTy::And => OpCode::And,
                        OpTy::Or => OpCode::Or,
                        OpTy::Xor => OpCode::Xor,

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

    pub fn resolve_local(&mut self, id: StrId, span: Span) -> Failible<usize> {
        self.valid_locals.get(&id).map(|i| *i).ok_or_else(|| {
            Diagnostic::build(Level::Error, DiagnosticType::Scope, span)
                .annotation(
                    Cow::Owned(format!(
                        "cannot resolve the variable `{}`",
                        self.c.interner.lookup(id)
                    )),
                    span,
                )
                .into()
        })
    }

    pub fn compile_assign(&mut self, id: Ident, e: Expr) -> Failible<()> {
        self.compile_expr(e)?;

        let local = self.c.write_interned(id.as_str());
        let stack_idx = self.resolve_local(local, id.1)?;

        self.write_bits(stack_idx.into(), OpCode::WriteSt, OpCode::WriteStLong);

        Ok(())
    }

    pub fn compile_dec(&mut self, id: Ident, e: Expr) -> Failible<()> {
        self.compile_expr(e)?;

        let local = self.c.write_interned(id.as_str());
        self.locals.push(Local {
            depth: self.scope_depth,
            name: local,
        });
        self.valid_locals.insert(local, self.locals.len() - 1);

        Ok(())
    }

    pub fn compile(&mut self, ast: Block) -> Failible<Chunk> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Expr(e) => self.compile_expr(e)?,
                StatementKind::DecAssign(id, e) => self.compile_dec(
                    id,
                    e.unwrap_or(Expr {
                        expr_kind: ExprKind::Null,
                        span: stmt.span,
                    }),
                )?,
                StatementKind::Assignment(id, e) => self.compile_assign(id, e)?,
                StatementKind::Input(id) => {
                    let strid = self.c.write_interned(&id.0);
                    let stid = Bits::from(self.resolve_local(strid, id.1)?);
                    self.write_bits(stid, OpCode::ReadLine, OpCode::ReadLineLong);
                }
                StatementKind::Print(e, no_newline) => {
                    self.compile_expr(e)?;
                    self.write_instr(
                        if !no_newline {
                            OpCode::PrtL
                        } else {
                            OpCode::Prt
                        },
                        stmt.span,
                    );
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
