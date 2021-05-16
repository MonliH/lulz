use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::mem;

use crate::{
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, Ident, OpTy, StatementKind},
    lolbc::{bits::Bits, Chunk, OpCode, Value},
    lolvm::StrId,
};

#[derive(Debug, PartialEq)]
enum RecentBlock {
    Function,
    Loop,
    Case,
}

#[derive(Debug)]
struct Local {
    depth: usize,
    name: StrId,
}

#[derive(Debug)]
pub struct BytecodeCompiler {
    c: Chunk,
    // map from interned string to it's stack position
    locals: SmallVec<[Local; 16]>,
    valid_locals: FxHashMap<StrId, usize>,
    scope_depth: usize,
    it: StrId,
    recent_block: RecentBlock,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        let mut new = Self {
            c: Chunk::new(),
            valid_locals: FxHashMap::default(),
            locals: SmallVec::default(),
            scope_depth: 0,
            it: StrId::default(),
            recent_block: RecentBlock::Function,
        };

        new.it = new.c.interner.intern("IT");
        new.declare_var(Ident("".into(), Span::default())).unwrap();
        new
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        let mut pops = 0;
        while self.locals.len() > 0
            && self.locals[self.locals.len() - pops - 1].depth > self.scope_depth
        {
            let str_id = self.locals.last().unwrap().name;
            self.valid_locals.remove(&str_id);
            pops += 1;
        }
        self.locals.truncate(self.locals.len() - pops);
        let bits = Bits::from(pops);
        self.write_bits(bits, OpCode::PopN, OpCode::PopNLong);
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

    fn read_var(&mut self, id: Ident) -> Failible<()> {
        let interned = self.c.interner.intern(id.0.as_str());
        if interned != self.it {
            let stid = self.resolve_local(interned, id.1)?;
            self.write_bits(stid.into(), OpCode::ReadSt, OpCode::ReadStLong);
        } else {
            self.write_instr(OpCode::ReadIt, id.1);
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Failible<()> {
        match expr.expr_kind {
            ExprKind::FunctionCall(id, args) => {
                let arg_len = args.len();
                if arg_len > 255 {
                    let span = expr.span;
                    return Err(Diagnostic::build(
                        Level::Error,
                        DiagnosticType::FunctionArgumentMany,
                        span,
                    )
                    .annotation(
                        Cow::Borrowed("a funkshon can only be called with up to 255 arguments"),
                        span,
                    )
                    .into());
                }
                self.read_var(id)?;
                for arg in args.into_iter() {
                    self.compile_expr(arg)?;
                }
                self.write_instr(OpCode::Call, expr.span);
                self.c.write_instr(arg_len as u8, expr.span);
            }

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
                self.read_var(id)?;
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
                        OpTy::Equal => OpCode::Equals,
                        OpTy::NotEq => {
                            self.write_instr(OpCode::Equals, expr.span);
                            self.write_instr(OpCode::Not, expr.span);
                            return Ok(());
                        }
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

    fn compile_assign(&mut self, id: Ident, e: Expr) -> Failible<()> {
        self.compile_expr(e)?;

        let local = self.c.write_interned(id.as_str());
        let stack_idx = self.resolve_local(local, id.1)?;

        self.write_bits(stack_idx.into(), OpCode::WriteSt, OpCode::WriteStLong);

        Ok(())
    }

    fn declare_intern(&mut self, id: StrId) -> Failible<()> {
        self.locals.push(Local {
            depth: self.scope_depth,
            name: id,
        });
        self.valid_locals.insert(id, self.locals.len() - 1);
        Ok(())
    }

    fn declare_var(&mut self, id: Ident) -> Failible<()> {
        let local = self.c.write_interned(id.as_str());
        self.locals.push(Local {
            depth: self.scope_depth,
            name: local,
        });
        self.valid_locals.insert(local, self.locals.len() - 1);
        Ok(())
    }

    fn compile_dec(&mut self, id: Ident, e: Expr) -> Failible<()> {
        self.compile_expr(e)?;
        self.declare_var(id)?;

        Ok(())
    }

    fn emit_jmp(&mut self, instr: OpCode, span: Span) -> usize {
        self.write_instr(instr, span);
        self.c.write_instr(0, span);
        self.c.write_instr(0, span);
        self.c.write_instr(0, span);
        self.c.write_instr(0, span);
        self.c.bytecode.len() - 4
    }

    fn patch_jmp(&mut self, offset: usize) {
        let jump = self.c.bytecode.len() - offset - 4;
        let bits = jump.to_le_bytes();
        self.c.bytecode[offset] = bits[0];
        self.c.bytecode[offset + 1] = bits[1];
        self.c.bytecode[offset + 2] = bits[2];
        self.c.bytecode[offset + 3] = bits[3];
    }

    pub fn compile_start(&mut self, ast: Block) -> Failible<()> {
        let span = ast.1;
        self.compile(ast)?;
        self.write_instr(OpCode::Return, span);
        Ok(())
    }

    fn compile(&mut self, ast: Block) -> Failible<()> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Break => {
                    if self.recent_block == RecentBlock::Function {
                        self.c.write_get_const(Value::Null, stmt.span);
                        self.write_instr(OpCode::Return, stmt.span);
                    }
                }
                StatementKind::Return(e) => {
                    self.compile_expr(e)?;
                    self.write_instr(OpCode::Return, stmt.span);
                }
                StatementKind::If(if_e, elif_es, else_e) => {
                    // 1: if a { b }
                    // 2: elif c { d }
                    // 3: elif e { f }
                    // 4: else { g }
                    //
                    // Compiles to;
                    // ld it
                    // jmpf 2 # jmp to next
                    // <b>
                    // jmpf 5
                    //
                    // 2:
                    // ld c
                    // jmpf 3 # jmp to next
                    // <d>
                    // jmpf 5
                    //
                    // 3:
                    // ld e
                    // jmpf 4 # jmp to next
                    // <f>
                    // jmpf 5
                    //
                    // 4:
                    // <g>
                    self.write_instr(OpCode::ReadIt, stmt.span);
                    let then_jmp = self.emit_jmp(OpCode::JmpFalse, stmt.span);
                    if let Some(true_block) = if_e {
                        self.begin_scope();
                        self.compile(true_block)?;
                        self.end_scope();
                    }
                    // prevent a fallthrough
                    let mut done_jmps = vec![self.emit_jmp(OpCode::Jmp, stmt.span)];
                    self.patch_jmp(then_jmp);

                    for elif in elif_es {
                        let span = elif.0.span;
                        self.compile_expr(elif.0)?;
                        let then_jmp = self.emit_jmp(OpCode::JmpFalse, stmt.span);
                        self.begin_scope();
                        self.compile(elif.1)?;
                        self.end_scope();
                        done_jmps.push(self.emit_jmp(OpCode::Jmp, span));
                        self.patch_jmp(then_jmp);
                    }

                    if let Some(else_block) = else_e {
                        self.begin_scope();
                        self.compile(else_block)?;
                        self.end_scope();
                    }
                    for done_jmp in done_jmps.into_iter() {
                        self.patch_jmp(done_jmp);
                    }
                }
                StatementKind::Expr(e) => {
                    let span = e.span;
                    self.compile_expr(e)?;
                    self.write_instr(OpCode::WriteIt, span);
                }
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
                StatementKind::FunctionDef(id, args, block) => {
                    if args.len() > 255 {
                        let span = args[0].1.combine(&args[args.len() - 1].1);
                        return Err(Diagnostic::build(
                            Level::Error,
                            DiagnosticType::FunctionArgumentMany,
                            span,
                        )
                        .annotation(
                            Cow::Borrowed("a funkshon's parameters are limited to 255 in length"),
                            span,
                        )
                        .into());
                    }
                    let skip_jmp = self.emit_jmp(OpCode::Jmp, stmt.span);
                    let function_pos = self.c.bytecode.len();
                    self.write_instr(OpCode::FnDef, id.1);
                    self.c.write_instr(args.len() as u8, id.1);

                    let old_locals = mem::replace(&mut self.valid_locals, FxHashMap::default());
                    let old_local_arr = mem::replace(&mut self.locals, SmallVec::new());
                    let interned = self.c.write_interned(id.0.as_str());
                    self.declare_intern(interned)?;
                    for arg in args {
                        self.declare_var(arg)?;
                    }

                    self.compile(block)?;
                    self.valid_locals = old_locals;
                    self.locals = old_local_arr;

                    self.write_instr(OpCode::ReadIt, id.1);
                    self.write_instr(OpCode::Return, id.1);

                    self.patch_jmp(skip_jmp);

                    self.c.write_get_const(Value::Fun(function_pos), id.1);
                    self.declare_intern(interned)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn take_chunk(&mut self) -> Chunk {
        mem::replace(&mut self.c, Chunk::default())
    }
}
