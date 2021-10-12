use std::mem;

use rustc_hash::FxHashSet;

use crate::{
    backend::CBuilder,
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, Ident, StatementKind},
};

use super::{Interner, StrId};

#[allow(dead_code)]
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
pub struct LowerCompiler {
    c: CBuilder,
    // map from interned string to it's stack position
    valid_locals: FxHashSet<StrId>,
    interner: Interner,
    it: StrId,
    recent_block: RecentBlock,
}

impl LowerCompiler {
    pub fn new(debug: bool) -> Self {
        let mut new = Self {
            c: CBuilder::new(debug),
            valid_locals: FxHashSet::default(),
            interner: Interner::default(),
            it: StrId::default(),
            recent_block: RecentBlock::Function,
        };

        new.it = new.interner.intern("IT");
        new
    }

    fn begin_scope(&mut self) {
        self.c.begin_scope();
    }
    fn end_scope(&mut self) {
        self.c.end_scope();
    }

    fn compile_func(
        &mut self,
        name: StrId,
        args: Vec<StrId>,
        block: Block,
        rec: bool,
        span: Span,
    ) -> Failible<()> {
        if args.len() > 255 {
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
        let args_len = args.len();

        let mut old_len = self.c.fns.len();
        mem::swap(&mut old_len, &mut self.c.fn_id);
        self.c.fns.push(String::new());

        let before_dec = self.c.write_dec();
        self.c.lol_value_ty();
        self.c.name(name);
        self.c.semi();
        self.c.fn_id = before_dec;

        self.c
            .debug_symbol("funkshon def", self.interner.lookup(name), span);
        self.c.fn_dec(name);
        self.begin_scope();
        let old_locals = mem::take(&mut self.valid_locals);
        self.c.args_check(args_len as u8);
        if rec {
            self.valid_locals.insert(name);
        }
        for (idx, argument) in args.into_iter().enumerate() {
            self.valid_locals.insert(argument);
            self.c.lol_value_ty();
            self.c.name(argument);
            self.c.ws(" = ");
            self.c.fn_values();
            self.c.wc('[');
            self.c.ws(&idx.to_string());
            self.c.wc(']');
            self.c.semi();
        }
        self.compile(block)?;

        self.c.ret();
        self.compile_expr(Expr {
            expr_kind: ExprKind::Null,
            span,
        })?;
        self.c.semi();

        self.end_scope();
        self.c.lol_value_ty();
        self.compile_assign(
            name,
            Expr {
                span,
                expr_kind: ExprKind::Function(name, args_len as u8),
            },
        )?;
        self.valid_locals = old_locals;
        if rec {
            self.valid_locals.insert(name);
        }
        mem::swap(&mut old_len, &mut self.c.fn_id);
        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Failible<()> {
        match expr.expr_kind {
            ExprKind::Cast(e, ty) => {}
            ExprKind::InterpStr(s, interps) => {}
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
                let span = id.1;
                let interned = self.intern(id);
                self.validate_local(interned, span)?;
                self.c
                    .debug_symbol("funkshon call", self.interner.lookup(interned), span);
                self.build_call(interned, args)?;
            }
            ExprKind::Function(fid, args) => self.c.function_ptr(fid, args),

            ExprKind::Float(f) => self.c.float(f),
            ExprKind::Int(i) => self.c.int(i),
            ExprKind::Bool(b) => self.c.bool(b),
            ExprKind::Null => self.c.null(),

            ExprKind::String(s) => {}

            ExprKind::Not(e) => {}

            ExprKind::Variable(id) => {
                let interned = self.intern(id.0.as_str());
                self.resolve_local(interned, id.1)?;
            }

            ExprKind::All(es) => {}
            ExprKind::Any(es) => {}
            ExprKind::Concat(es) => {}

            ExprKind::Operator(op, e1, e2) => {}
        }
        Ok(())
    }

    fn validate_local(&mut self, id: StrId, span: Span) -> Failible<()> {
        if id == self.it {
            return Ok(());
        }
        self.valid_locals.get(&id).map(|i| *i).ok_or_else(|| {
            Diagnostics::from(
                Diagnostic::build(Level::Error, DiagnosticType::Scope, span).annotation(
                    Cow::Owned(format!(
                        "cannot resolve the variable `{}`",
                        self.interner.lookup(id)
                    )),
                    span,
                ),
            )
        })?;
        Ok(())
    }

    fn resolve_local(&mut self, id: StrId, span: Span) -> Failible<()> {
        if id == self.it {
            self.c.it();
        } else {
            self.c.name(id);
            self.validate_local(id, span)?;
        }

        Ok(())
    }

    fn intern<T>(&mut self, id: T) -> StrId
    where
        T: AsRef<str>,
    {
        self.interner.intern(id.as_ref())
    }

    pub fn compile_start(&mut self, ast: Block) -> Failible<()> {
        let interned = self.intern("main_lulz");
        self.c.main_fn = format!("lol_{}_fn", interned.get_id());
        self.compile_func(interned, Vec::new(), ast, false, Span::default())?;
        self.c.stdlib();
        Ok(())
    }

    fn compile_assign(&mut self, id: StrId, expr: Expr) -> Failible<()> {
        self.valid_locals.insert(id);
        self.c.name(id);
        self.c.ws(" = ");
        self.compile_expr(expr)?;
        self.c.semi();
        Ok(())
    }

    fn build_call(&mut self, id: StrId, args: Vec<Expr>) -> Failible<()> {
        self.c.ws("lol_call");
        self.c.wc('(');
        self.c.ws(&args.len().to_string());
        self.c.ws(", ");
        self.c.name(id);
        self.c.ws(", ");
        if !args.is_empty() {
            self.c.ws("(LolValue[]){");
            let mut args = args.into_iter();
            if let Some(arg) = args.next() {
                self.compile_expr(arg)?;
            }
            for arg in args {
                self.c.ws(", ");
                self.compile_expr(arg)?;
            }
            self.c.wc('}');
        } else {
            self.c.ws("NULL");
        }
        self.c.wc(')');
        Ok(())
    }

    fn compile(&mut self, ast: Block) -> Failible<()> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Break => {}
                StatementKind::Return(e) => {
                    self.c.ret();
                    self.compile_expr(e)?;
                    self.c.semi();
                }
                StatementKind::If(if_e, elif_es, else_e) => {}
                StatementKind::Expr(e) => {
                    self.compile_expr(e)?;
                    self.c.semi();
                }
                StatementKind::DecAssign(id, e) => {
                    self.c.debug_symbol("declare", id.0.as_str(), stmt.span);
                    self.c.lol_value_ty();
                    let ident = self.intern(id);
                    self.compile_assign(
                        ident,
                        e.unwrap_or(Expr {
                            span: stmt.span,
                            expr_kind: ExprKind::Null,
                        }),
                    )?;
                }
                StatementKind::Assignment(id, e) => {
                    self.c.debug_symbol("assign", id.0.as_str(), stmt.span);
                    let ident = self.intern(id);
                    self.compile_assign(ident, e)?;
                }
                StatementKind::Input(id) => {}
                StatementKind::Print(e, no_newline) => {}
                StatementKind::FunctionDef(id, args, block) => {
                    let fn_name = self.intern(id);
                    let args = args.into_iter().map(|arg| self.intern(arg)).collect();
                    self.compile_func(fn_name, args, block, true, stmt.span)?
                }
                StatementKind::MutCast(id, ty) => {}
                _ => {}
            }
        }

        Ok(())
    }

    pub fn get_str(self) -> String {
        self.c.output()
    }
}
