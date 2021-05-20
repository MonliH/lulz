use std::mem;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    backend::CBuilder,
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, Ident, OpTy, StatementKind},
};

use super::{Interner, StrId};

#[derive(Debug, PartialEq)]
enum RecentBlock {
    Function,
    Loop,
    Case,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ValueTy {
    Function(u8),
    Value,
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
    valid_locals: FxHashMap<StrId, ValueTy>,
    locals: SmallVec<[(StrId, usize); 16]>,
    depth: usize,
    interner: Interner,
    it: StrId,
    recent_block: RecentBlock,
}

impl LowerCompiler {
    pub fn new(debug: bool) -> Self {
        let mut new = Self {
            c: CBuilder::new(debug),
            valid_locals: FxHashMap::default(),
            locals: SmallVec::new(),
            depth: 0,
            interner: Interner::default(),
            it: StrId::default(),
            recent_block: RecentBlock::Function,
        };

        new.it = new.interner.intern("IT");
        new
    }

    fn begin_scope(&mut self) {
        self.c.begin_scope();
        self.depth += 1;
    }

    fn end_scope(&mut self) {
        self.depth -= 1;
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].1 > self.depth {
            let str_id = self.locals[self.locals.len() - 1].0;
            self.valid_locals.remove(&str_id);
            self.locals.pop();
        }
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
        self.c.fn_dec(name, &args);
        self.c.semi();
        self.c.fn_id = before_dec;

        self.c
            .debug_symbol("funkshon def", self.interner.lookup(name), span);
        self.c.fn_dec(name, &args);
        self.begin_scope();
        let old_locals = mem::take(&mut self.valid_locals);
        let function_val = ValueTy::Function(args_len as u8);
        if rec {
            self.valid_locals.insert(name, function_val);
            self.locals.push((name, self.depth));
        }
        for argument in args.into_iter() {
            self.valid_locals.insert(argument, ValueTy::Value);
            self.locals.push((argument, self.depth));
        }
        self.compile(block)?;

        self.c.ret();
        self.c.it();
        self.c.semi();

        self.end_scope();
        self.valid_locals = old_locals;
        if rec {
            self.valid_locals.insert(name, function_val);
            self.locals.push((name, self.depth));
        }
        mem::swap(&mut old_len, &mut self.c.fn_id);
        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Failible<()> {
        match expr.expr_kind {
            ExprKind::Cast(e, ty) => {
                self.c.cast(ty);
                self.c.wc('(');
                self.compile_expr(*e)?;
                self.c.wc(')');
            }
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
                        Cow::Borrowed("a FUNKSHON can only be called with up to 255 arguments"),
                        span,
                    )
                    .into());
                }
                let span = id.1;
                let interned = self.intern(id);
                let arity = self.validate_fn(interned, span)?;
                if (arg_len as u8) != arity {
                    let span = expr.span;
                    return Err(Diagnostic::build(
                        Level::Error,
                        DiagnosticType::FunctionArgumentMany,
                        span,
                    )
                    .annotation(
                        Cow::Owned(format!(
                            "this FUNKSHON should take {} arugument(s), but it recived {}",
                            arity, arg_len,
                        )),
                        span,
                    )
                    .into());
                }
                self.c
                    .debug_symbol("funkshon call", self.interner.lookup(interned), span);
                self.build_call(interned, args, expr.span)?;
            }
            ExprKind::Function(fid, args) => self.c.function_ptr(fid),

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

            ExprKind::Operator(op, e1, e2) => {
                let fn_name = match op {
                    OpTy::Add => "lol_add",
                    OpTy::Sub => "lol_sub",
                    OpTy::Mul => "lol_mul",
                    OpTy::Div => "lol_div",
                    OpTy::Mod => "lol_mod",

                    OpTy::Min => "lol_min",
                    OpTy::Max => "lol_max",

                    OpTy::And => "lol_and",
                    OpTy::Or => "lol_or",
                    OpTy::Xor => "lol_xor",

                    OpTy::Equal => "lol_eq",
                    OpTy::NotEq => "lol_neq",

                    OpTy::GT => "lol_gt",
                    OpTy::LT => "lol_lt",
                    OpTy::GTE => "lol_gte",
                    OpTy::LTE => "lol_lte",
                };
                self.c
                    .debug_symbol("operator", fn_name, e1.span.combine(&e2.span));
                self.c.ws(fn_name);
                self.c.wc('(');
                self.compile_expr(*e1)?;
                self.c.ws(", ");
                self.compile_expr(*e2)?;
                self.c.ws(", ");
                self.c.span(expr.span);
                self.c.wc(')');
            }
        }
        Ok(())
    }

    fn validate_fn(&mut self, id: StrId, span: Span) -> Failible<u8> {
        if let ValueTy::Function(arity) =
            self.valid_locals.get(&id).map(|i| *i).ok_or_else(|| {
                Diagnostics::from(
                    Diagnostic::build(Level::Error, DiagnosticType::Scope, span).annotation(
                        Cow::Owned(format!(
                            "cannot resolve the FUNKSHON `{}`",
                            self.interner.lookup(id)
                        )),
                        span,
                    ),
                )
            })?
        {
            Ok(arity)
        } else {
            Err(Diagnostics::from(
                Diagnostic::build(Level::Error, DiagnosticType::Scope, span).annotation(
                    Cow::Owned(format!("`{}` is not a FUNKSHON", self.interner.lookup(id))),
                    span,
                ),
            ))
        }
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
        self.c.name(id);
        self.c.ws(" = ");
        self.compile_expr(expr)?;
        self.c.semi();
        Ok(())
    }

    fn build_call(&mut self, id: StrId, args: Vec<Expr>, span: Span) -> Failible<()> {
        self.c.name(id);
        self.c.ws("_fn(");
        let mut args = args.into_iter();
        if let Some(arg) = args.next() {
            self.compile_expr(arg)?;
        }
        for arg in args {
            self.c.ws(", ");
            self.compile_expr(arg)?;
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
                StatementKind::If(then, elif_es, else_block) => {
                    self.c.ws("if (lol_to_bool(");
                    self.c.it();
                    self.c.ws("))");
                    self.begin_scope();
                    if let Some(th) = then {
                        self.compile(th)?;
                    }
                    self.end_scope();

                    self.c.ws("else");
                    self.begin_scope();
                    if let Some(else_bl) = else_block {
                        self.compile(else_bl)?;
                    }
                    self.end_scope();
                }
                StatementKind::Expr(e) => {
                    self.c.it();
                    self.c.ws(" = ");
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
                    self.valid_locals.insert(ident, ValueTy::Value);
                    self.locals.push((ident, self.depth));
                }
                StatementKind::Assignment(id, e) => {
                    self.c.debug_symbol("assign", id.0.as_str(), stmt.span);
                    let ident = self.intern(id);
                    self.compile_assign(ident, e)?;
                }
                StatementKind::Input(id) => {}
                StatementKind::Print(e, no_newline) => {
                    let fn_name = if no_newline {
                        "lol_print"
                    } else {
                        "lol_println"
                    };
                    self.c.ws(fn_name);
                    self.c.wc('(');
                    self.compile_expr(e)?;
                    self.c.wc(')');
                    self.c.semi();
                }
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
