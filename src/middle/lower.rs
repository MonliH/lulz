use std::mem;

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use crate::{
    backend::CBuilder,
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, OpTy, StatementKind},
};

use super::{Interner, StrId};

#[derive(Debug, PartialEq, Copy, Clone)]
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
pub struct LowerCompiler {
    c: CBuilder,

    // map from interned string to it's stack position
    valid_locals: FxHashMap<StrId, (ValueTy, usize)>,
    locals: SmallVec<[(StrId, usize); 16]>,
    overwritten: SmallVec<[Vec<(StrId, (ValueTy, usize))>; 8]>,

    depth: usize,
    interner: Interner,
    it: StrId,
    recent_block: RecentBlock,

    deced_dyns: FxHashSet<StrId>,
}

impl LowerCompiler {
    pub fn new(debug: bool) -> Self {
        let mut new = Self {
            c: CBuilder::new(debug),

            valid_locals: FxHashMap::default(),
            locals: SmallVec::new(),
            overwritten: SmallVec::new(),

            depth: 0,
            interner: Interner::default(),
            it: StrId::default(),
            recent_block: RecentBlock::Function,
            deced_dyns: FxHashSet::default(),
        };

        new.it = new.interner.intern("IT");
        new
    }

    fn begin_scope(&mut self) {
        self.c.begin_scope();
        self.depth += 1;
        self.overwritten.push(Vec::new());
    }

    fn end_scope(&mut self) {
        self.depth -= 1;
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].1 > self.depth {
            let str_id = self.locals[self.locals.len() - 1].0;
            self.valid_locals.remove(&str_id);
            self.locals.pop();
        }
        self.c.end_scope();
        let additionals = self.overwritten.pop().unwrap();
        self.valid_locals.extend(additionals);
    }

    fn insert_local(&mut self, name: StrId, value: ValueTy) {
        if let Some(val) = self.valid_locals.get(&name) {
            let len = self.overwritten.len() - 1;
            self.overwritten[len].push((name, *val));
        }
        self.valid_locals.insert(name, (value, self.depth));
        self.locals.push((name, self.depth));
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
            self.insert_local(name, function_val)
        }
        for argument in args.into_iter() {
            self.insert_local(argument, ValueTy::Value);
        }
        self.compile(block)?;

        self.c.ret();
        self.c.it();
        self.c.semi();

        self.end_scope();
        self.valid_locals = old_locals;
        if rec {
            self.insert_local(name, function_val)
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
                self.c
                    .debug_symbol("funkshon call", self.interner.lookup(interned), span);
                if let ValueTy::Function(arity) = self.validate_local(interned, span)? {
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
                    self.build_call(interned, args)?;
                } else {
                    self.build_dynamic_call(interned, args, expr.span)?;
                }
            }

            ExprKind::Float(f) => self.c.float(f),
            ExprKind::Int(i) => self.c.int(i),
            ExprKind::Bool(b) => self.c.bool(b),
            ExprKind::Null => self.c.null(),

            ExprKind::String(s) => self.c.string_lit(&s, s.len()),

            ExprKind::Not(e) => {
                self.c.ws("lol_not");
                self.c.wc('(');
                self.c.wc(')');
            }

            ExprKind::Variable(id) => {
                let interned = self.intern(id.0.as_str());
                self.resolve_local(interned, id.1)?;
            }

            ExprKind::All(es) => {}
            ExprKind::Any(es) => {}
            ExprKind::Concat(es) => {
                self.c.ws("OBJ_VALUE(lol_alloc_stack_str(lol_concat_str(");
                self.c.ws(&es.len().to_string());
                for e in es {
                    self.c.ws(", ");
                    self.compile_expr(e)?;
                }
                self.c.ws(")))");
            }

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

    fn validate_local(&mut self, id: StrId, span: Span) -> Failible<ValueTy> {
        if id == self.it {
            return Ok(ValueTy::Value);
        }
        self.valid_locals.get(&id).map(|i| (*i).0).ok_or_else(|| {
            Diagnostics::from(
                Diagnostic::build(Level::Error, DiagnosticType::Scope, span).annotation(
                    Cow::Owned(format!(
                        "cannot resolve the variable `{}`",
                        self.interner.lookup(id)
                    )),
                    span,
                ),
            )
        })
    }

    fn build_dynamic_fn(&mut self, id: StrId, arity: u8) {
        self.c.ws(&format!(
            include_str!("../clib/dyn_function.clol"),
            id.get_id(),
            arity,
        ));

        for i in 0..arity {
            self.c.ws(&format!("LolValue arg_{i} = values[{i}]", i = i));
            self.c.semi();
        }

        self.c.ws(&format!("lol_{}_fn(", id.get_id()));
        let mut args = (0..arity).into_iter();
        if let Some(n) = args.next() {
            self.c.ws("arg_");
            self.c.ws(&n.to_string());
        }

        for arg in args {
            self.c.ws(", arg_");
            self.c.ws(&arg.to_string());
        }

        self.c.wc(')');
        self.c.semi();
        self.c.ws("}\n");
    }

    fn resolve_local(&mut self, id: StrId, span: Span) -> Failible<()> {
        if id == self.it {
            self.c.it();
        } else {
            if let ValueTy::Function(arity) = self.validate_local(id, span)? {
                if !self.deced_dyns.contains(&id) {
                    self.deced_dyns.insert(id);
                    let before_dec = self.c.write_dec();
                    self.c.fn_dec_dyn(id);
                    self.c.fn_id = before_dec;

                    let mut old_len = self.c.fns.len();
                    mem::swap(&mut old_len, &mut self.c.fn_id);
                    self.c.fns.push(String::new());
                    self.build_dynamic_fn(id, arity);
                    mem::swap(&mut old_len, &mut self.c.fn_id);
                }

                self.c.function_ptr(id)
            } else {
                self.c.name(id);
            }
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

    fn compile_assign(&mut self, id: StrId, expr: Expr, span: Span) -> Failible<()> {
        self.resolve_local(id, span)?;
        self.c.ws(" = ");
        self.compile_expr(expr)?;
        self.c.semi();
        Ok(())
    }

    fn compile_dec(&mut self, id: StrId, expr: Expr) -> Failible<()> {
        self.c.name(id);
        self.c.ws(" = ");
        self.compile_expr(expr)?;
        self.c.semi();
        self.insert_local(id, ValueTy::Value);
        Ok(())
    }

    fn build_call(&mut self, id: StrId, args: Vec<Expr>) -> Failible<()> {
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

    fn build_dynamic_call(&mut self, id: StrId, args: Vec<Expr>, span: Span) -> Failible<()> {
        self.c.ws("lol_call(");
        self.c.ws(&args.len().to_string());
        self.c.ws(", ");
        self.c.name(id);
        self.c.ws(", ");
        if !args.is_empty() {
            self.c.wc('(');
            self.c.lol_value_ty();
            self.c.ws("[]){");
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
        self.c.ws(", ");
        self.c.span(span);
        self.c.ws(")");
        Ok(())
    }

    fn compile(&mut self, ast: Block) -> Failible<()> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Break => match self.recent_block {
                    RecentBlock::Function => {
                        self.c.ws("return ");
                        self.c.null();
                        self.c.semi();
                    }
                    _ => {}
                },
                StatementKind::Return(e) => {
                    self.c.ret();
                    self.compile_expr(e)?;
                    self.c.semi();
                }
                StatementKind::If(then, elif_es, else_block) => {
                    self.c.ws("if (lol_to_bool(");
                    self.c.it();
                    self.c.ws(")) ");
                    self.begin_scope();
                    if let Some(th) = then {
                        self.compile(th)?;
                    }
                    self.end_scope();

                    for elif in elif_es {
                        self.c.ws("else if (lol_to_bool(");
                        self.compile_expr(elif.0)?;
                        self.c.ws(")) ");
                        self.begin_scope();
                        self.compile(elif.1)?;
                        self.end_scope();
                    }

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
                    if let Some((_, depth)) = self.valid_locals.get(&ident) {
                        if *depth == self.depth {
                            return Err(Diagnostic::build(
                                Level::Error,
                                DiagnosticType::Scope,
                                stmt.span,
                            )
                            .annotation(
                                Cow::Owned(format!(
                                    "variable `{}` already declared in this scope",
                                    self.interner.lookup(ident)
                                )),
                                stmt.span,
                            )
                            .into());
                        }
                    }
                    self.compile_dec(
                        ident,
                        e.unwrap_or(Expr {
                            span: stmt.span,
                            expr_kind: ExprKind::Null,
                        }),
                    )?;
                }
                StatementKind::Assignment(id, e) => {
                    self.c.debug_symbol("assign", id.0.as_str(), stmt.span);
                    let span = id.1;
                    let ident = self.intern(id);
                    self.compile_assign(ident, e, span)?;
                }
                StatementKind::Input(id) => {
                    self.c.ws("lol_readline(&");
                    let ident = self.intern(id);
                    self.c.name(ident);
                    self.c.wc(')');
                    self.c.semi();
                }
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
                    let prev = mem::replace(&mut self.recent_block, RecentBlock::Function);
                    let fn_name = self.intern(id);
                    let args = args.into_iter().map(|arg| self.intern(arg)).collect();
                    self.compile_func(fn_name, args, block, true, stmt.span)?;
                    self.recent_block = prev;
                }
                StatementKind::MutCast(id, ty) => {
                    self.c.debug_symbol("mut cast", id.0.as_str(), stmt.span);
                    let span = id.1;
                    let ident = self.intern(id.clone());
                    self.compile_assign(
                        ident,
                        Expr {
                            span: stmt.span,
                            expr_kind: ExprKind::Cast(
                                Box::new(Expr {
                                    span: stmt.span,
                                    expr_kind: ExprKind::Variable(id),
                                }),
                                ty,
                            ),
                        },
                        span,
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn get_str(self) -> String {
        self.c.output()
    }
}
