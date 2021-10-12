use std::{iter::once, mem};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    backend::CBuilder,
    diagnostics::prelude::*,
    frontend::ast::{Block, Expr, ExprKind, LolTy, OpTy, StatementKind, UnOpTy},
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
    global_builtins: FxHashMap<StrId, ValueTy>,
    valid_locals: FxHashMap<StrId, (ValueTy, usize)>,
    locals: Vec<(StrId, usize)>,
    overwritten: Vec<Vec<(StrId, (ValueTy, usize))>>,

    depth: usize,
    // map depth to fn_depth
    fn_depth_map: FxHashMap<usize, usize>,
    upvalues: Vec<(FxHashMap<StrId, usize>, StrId)>,
    fn_depth: usize,

    interner: Interner,
    it: StrId,
    recent_block: RecentBlock,

    deced_dyns: FxHashSet<StrId>,
    case_id: usize,
    end_case_id: usize,
}

impl LowerCompiler {
    pub fn new(debug: bool) -> Self {
        let mut new = Self {
            c: CBuilder::new(debug),

            global_builtins: FxHashMap::default(),
            valid_locals: FxHashMap::default(),
            locals: Vec::new(),
            overwritten: Vec::new(),

            depth: 0,
            fn_depth: 0,
            fn_depth_map: FxHashMap::default(),
            upvalues: vec![(FxHashMap::default(), StrId::default())],

            interner: Interner::default(),
            it: StrId::default(),
            recent_block: RecentBlock::Function,
            deced_dyns: FxHashSet::default(),

            case_id: 0,
            end_case_id: 0,
        };

        new.it = new.interner.intern("IT");
        new
    }

    fn begin_scope(&mut self) {
        self.c.begin_scope();
        self.depth += 1;
        self.fn_depth_map.insert(self.depth, self.fn_depth);
        self.overwritten.push(Vec::new());
    }

    fn end_scope(&mut self) {
        self.fn_depth_map.remove(&self.depth);
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
            return Err(
                Diagnostic::build(DiagnosticType::FunctionArgumentMany, span)
                    .annotation(
                        Cow::Borrowed("a funkshon's parameters are limited to 255 in length"),
                        span,
                    )
                    .into(),
            );
        }
        let args_len = args.len();

        let mut old_len = self.c.fns.len();
        mem::swap(&mut old_len, &mut self.c.fn_id);
        self.c.fns.push(String::new());

        self.fn_depth += 1;
        self.upvalues.push((FxHashMap::default(), name));
        self.begin_scope();
        let function_val = ValueTy::Function(args_len as u8);
        if rec {
            self.insert_local(name, function_val)
        }
        for argument in args.iter() {
            self.insert_local(*argument, ValueTy::Value);
        }
        println!("=== {} ===", self.interner.lookup(name));
        self.compile(block)?;

        self.c.ret();
        self.c.it();
        self.c.semi();

        self.end_scope();
        self.fn_depth -= 1;
        let upvalues = self.upvalues.pop().unwrap();

        let old = mem::take(&mut self.c.fns[self.c.fn_id]);

        if upvalues.0.is_empty() {
            let before_dec = self.c.write_dec();
            self.c.fn_dec(name, &args);
            self.c.semi();
            self.c.fn_id = before_dec;

            self.c
                .debug_symbol("funkshon def", self.interner.lookup(name), span);
            self.c.fn_dec(name, &args);
            self.c.ws(&old);
            if rec {
                self.insert_local(name, function_val)
            }
            mem::swap(&mut old_len, &mut self.c.fn_id);
        } else {
            let fn_name = self.c.closure_name(
                self.upvalues
                    .iter()
                    .map(|(_, fn_name)| *fn_name)
                    .chain(once(name)),
            );
            let before_dec = self.c.write_dec();
            self.c.dec_closure(&fn_name);
            self.c.fn_id = before_dec;
            self.c
                .debug_symbol("closure def", self.interner.lookup(name), span);
            self.c.def_closure(args.len(), &fn_name);
            for (i, arg) in args.iter().enumerate() {
                self.c.lol_value_ty();
                self.c.name(*arg);
                self.c.ws(" = args[");
                self.c.ws(&i.to_string());
                self.c.wc(']');
                self.c.semi();
            }
            self.c.ws(&old[1..]);
            self.insert_local(name, ValueTy::Value);
            mem::swap(&mut old_len, &mut self.c.fn_id);
            self.c.lol_value_ty();
            self.c.name(name);
            self.c.ws(" = ");
            self.c
                .ws("OBJ_VALUE(lol_alloc_stack_closure(lol_init_closure(");
            self.c.ws(&fn_name);
            self.c.comma();
            self.c.ws(&upvalues.0.len().to_string());
            for absorbed in upvalues.0 {
                self.c.comma();
                self.c.ws("(LolValue*)&");
                self.resolve_local(absorbed.0, Span::default())?;
            }
            self.c.ws(")))");
            self.c.semi();
        }

        Ok(())
    }

    fn compile_str_lit(&mut self, s: &str) {
        self.c.wc('"');
        self.c.ws(s);
        self.c.wc('"');
    }

    fn string_lit(&mut self, s: &str, len: usize) {
        self.c.ws("OBJ_VALUE(");
        self.c.ws("lol_alloc_lit_str((char*)");
        self.compile_str_lit(s);
        self.c.comma();
        self.c.ws(&len.to_string());
        self.c.ws("))");
    }

    fn compile_expr(&mut self, expr: Expr) -> Failible<()> {
        match expr.expr_kind {
            ExprKind::Cast(e, ty) => {
                self.c.cast(ty);
                self.c.wc('(');
                self.compile_expr(*e)?;
                self.c.wc(')');
            }
            ExprKind::InterpStr(s, interps) => {
                self.c.ws("OBJ_VALUE(lol_alloc_stack_str(lol_interp_str(");
                self.c.ws(&interps.len().to_string());
                let mut running_str = &s[..];
                let mut idx = 0;
                for interp in interps {
                    let interned = self.intern(interp.1);
                    let (fragment, new_str) = running_str.split_at(interp.0 - idx);
                    running_str = new_str;
                    idx = interp.0;
                    self.c.ws(", (size_t)");
                    self.c.ws(&fragment.len().to_string());
                    self.c.comma();
                    self.compile_str_lit(fragment);
                    self.c.comma();
                    self.resolve_local(interned, interp.2)?;
                }
                self.c.ws(", (size_t)");
                self.c.ws(&running_str.len().to_string());
                self.c.comma();
                self.compile_str_lit(running_str);
                self.c.ws(")))");
            }
            ExprKind::FunctionCall(id, args) => {
                let arg_len = args.len();
                if arg_len > 255 {
                    let span = expr.span;
                    return Err(
                        Diagnostic::build(DiagnosticType::FunctionArgumentMany, span)
                            .annotation(
                                Cow::Borrowed(
                                    "a FUNKSHON can only be called with up to 255 arguments",
                                ),
                                span,
                            )
                            .into(),
                    );
                }
                let span = id.1;
                let interned = self.intern(id);
                self.c
                    .debug_symbol("funkshon call", self.interner.lookup(interned), span);
                if let Ok(ValueTy::Function(arity)) = self.validate_local(interned, span)? {
                    if (arg_len as u8) != arity {
                        let span = expr.span;
                        return Err(
                            Diagnostic::build(DiagnosticType::FunctionArgumentMany, span)
                                .annotation(
                                    Cow::Owned(format!(
                                "this FUNKSHON should take {} arugument(s), but it recived {}",
                                arity, arg_len,
                            )),
                                    span,
                                )
                                .into(),
                        );
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

            ExprKind::String(s) => self.string_lit(&s, s.len()),

            ExprKind::UnaryOp(opty, e) => {
                self.c.ws(match opty {
                    UnOpTy::Not => "lol_not",
                    UnOpTy::Length => "lol_length",
                });
                self.c.wc('(');
                self.compile_expr(*e)?;
                self.c.wc(')');
            }

            ExprKind::Variable(id) => {
                let interned = self.intern(id.0.as_str());
                self.resolve_local(interned, id.1)?;
            }

            ExprKind::All(es) => {
                self.c.ws("lol_all(");
                self.c.ws(&es.len().to_string());
                for e in es {
                    self.c.ws(", ");
                    self.compile_expr(e)?;
                }
                self.c.wc(')');
            }
            ExprKind::Any(es) => {
                self.c.ws("lol_any(");
                self.c.ws(&es.len().to_string());
                for e in es {
                    self.c.ws(", ");
                    self.compile_expr(e)?;
                }
                self.c.wc(')');
            }
            ExprKind::Concat(es) => {
                self.c.ws("OBJ_VALUE(lol_alloc_stack_str(lol_concat_str(");
                self.c.ws(&es.len().to_string());
                for e in es {
                    self.c.comma();
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
                self.c.comma();
                self.compile_expr(*e2)?;
                self.c.comma();
                self.c.span(expr.span);
                self.c.wc(')');
            }
            ExprKind::List(exprs) => {
                self.c.ws("lol_vec_lit(");
                let mut cap = 1;
                while cap < exprs.len() {
                    cap <<= 2;
                }
                self.c.ws("(size_t)");
                self.c.ws(&cap.to_string());
                self.c.comma();
                self.c.ws("(size_t)");
                self.c.ws(&exprs.len().to_string());
                for expr in exprs {
                    self.c.comma();
                    self.compile_expr(expr)?;
                }
                self.c.wc(')');
            }
            ExprKind::GetItem(e, idx) => match idx {
                Ok(idx_e) => {
                    self.c.ws("lol_vec_index(");
                    self.compile_expr(*e)?;
                    self.c.comma();
                    self.compile_expr(*idx_e)?;
                    self.c.comma();
                    self.c.span(expr.span);
                    self.c.wc(')');
                }
                Err(is_front) => {
                    self.c.ws(if is_front {
                        "lol_vec_first"
                    } else {
                        "lol_vec_last"
                    });
                    self.c.wc('(');
                    self.compile_expr(*e)?;
                    self.c.comma();
                    self.c.span(expr.span);
                    self.c.wc(')');
                }
            },
        }
        Ok(())
    }

    fn validate_local(&mut self, id: StrId, span: Span) -> Failible<Result<ValueTy, usize>> {
        if let Some(val) = self.global_builtins.get(&id) {
            return Ok(Ok(*val));
        }
        if id == self.it {
            return Ok(Ok(ValueTy::Value));
        }
        let res = self.valid_locals.get(&id);
        match res {
            Some((ty, depth)) => {
                let ty_depth = self.fn_depth_map.get(&depth).unwrap();
                if *ty_depth == self.fn_depth {
                    return Ok(Ok(*ty));
                }
                match ty {
                    ValueTy::Function(..) => Ok(Ok(*ty)),
                    ValueTy::Value => {
                        if !self.upvalues[self.fn_depth].0.contains_key(&id) {
                            for d in (*ty_depth + 1)..=self.fn_depth {
                                let idx = self.upvalues[d].0.len();
                                self.upvalues[d].0.insert(id, idx);
                            }
                        }
                        Ok(Err(self.upvalues[self.fn_depth].0[&id]))
                    }
                }
            }
            None => Err(Diagnostics::from(
                Diagnostic::build(DiagnosticType::Scope, span).annotation(
                    Cow::Owned(format!(
                        "cannot resolve the variable `{}`",
                        self.interner.lookup(id)
                    )),
                    span,
                ),
            )),
        }
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

        self.c.ws("return ");
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
            let value = self.validate_local(id, span)?;
            match value {
                Ok(ValueTy::Function(arity)) => {
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
                }
                Err(id) => {
                    self.c.upvalue(id);
                }
                Ok(ValueTy::Value) => {
                    self.c.name(id);
                }
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

    fn compile_fn_builtin(&mut self, name: &'static str, arity: u8) -> StrId {
        let id = self.intern(name);
        self.global_builtins.insert(id, ValueTy::Function(arity));
        id
    }

    fn compile_builtins(&mut self) {
        let uppin = self.compile_fn_builtin("UPPIN", 1);
        let nerfin = self.compile_fn_builtin("NERFIN", 1);

        let mut old_len = self.c.fns.len();
        mem::swap(&mut old_len, &mut self.c.fn_id);
        self.c.fns.push(String::new());
        self.c.ws(&format!(
            include_str!("../clib/builtins.clol"),
            nerfin = nerfin.get_id(),
            uppin = uppin.get_id()
        ));
        mem::swap(&mut old_len, &mut self.c.fn_id);
    }

    pub fn compile_start(&mut self, ast: Block) -> Failible<()> {
        let interned = self.intern("main_lulz");
        self.c.main_fn = format!("lol_{}_fn", interned.get_id());
        self.compile_builtins();
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
            self.c.comma();
            self.compile_expr(arg)?;
        }
        self.c.wc(')');
        Ok(())
    }

    fn build_dynamic_call(&mut self, id: StrId, args: Vec<Expr>, span: Span) -> Failible<()> {
        self.c.ws("lol_call(");
        self.c.ws(&args.len().to_string());
        self.c.comma();
        self.resolve_local(id, span)?;
        self.c.comma();
        if !args.is_empty() {
            self.c.wc('(');
            self.c.lol_value_ty();
            self.c.ws("[]){");
            let mut args = args.into_iter();
            if let Some(arg) = args.next() {
                self.compile_expr(arg)?;
            }
            for arg in args {
                self.c.comma();
                self.compile_expr(arg)?;
            }
            self.c.wc('}');
        } else {
            self.c.ws("NULL");
        }
        self.c.comma();
        self.c.span(span);
        self.c.ws(")");
        Ok(())
    }

    fn default_expr(&mut self, ty: LolTy, span: Span) -> Expr {
        Expr {
            span,
            expr_kind: ty.default_expr_kind(),
        }
    }

    fn compile(&mut self, ast: Block) -> Failible<()> {
        for stmt in ast.0.into_iter() {
            match stmt.statement_kind {
                StatementKind::Loop {
                    block_name: _,
                    fn_id,
                    block,
                } => {
                    let prev_block = mem::replace(&mut self.recent_block, RecentBlock::Loop);
                    match fn_id {
                        Some((fn_name, var_name, pred)) => {
                            // for loop
                            let inc = self.intern(var_name.clone());
                            self.insert_local(inc, ValueTy::Value);
                            self.c.ws("for (");
                            self.c.lol_value_ty();
                            self.c.name(inc);
                            self.c.ws(" = INT_VALUE(0); ");
                            if let Some((till, e)) = pred {
                                if till {
                                    self.c.wc('!');
                                }
                                self.c.ws("lol_to_bool(");
                                self.compile_expr(e)?;
                                self.c.wc(')');
                            }
                            self.c.ws("; ");
                            self.c.name(inc);
                            self.c.ws(" = ");
                            let span = var_name.1;
                            self.compile_expr(Expr {
                                span: fn_name.1,
                                expr_kind: ExprKind::FunctionCall(
                                    fn_name,
                                    vec![Expr {
                                        span: span,
                                        expr_kind: ExprKind::Variable(var_name.clone()),
                                    }],
                                ),
                            })?;
                            self.c.wc(')');
                            self.begin_scope();
                            self.compile(block)?;
                            self.end_scope();
                        }
                        None => {
                            // while loop
                            self.c.ws("while (1)");
                            self.begin_scope();
                            self.compile(block)?;
                            self.end_scope();
                        }
                    }
                    self.recent_block = prev_block;
                }
                StatementKind::Break => match self.recent_block {
                    RecentBlock::Function => {
                        self.c.ws("return ");
                        self.c.null();
                        self.c.semi();
                    }
                    RecentBlock::Case => {
                        self.c.ws("goto ");
                        self.c.lol_case_jmp(self.end_case_id);
                        self.c.semi();
                    }
                    RecentBlock::Loop => {
                        self.c.ws("break");
                        self.c.semi();
                    }
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
                            return Err(Diagnostic::build(DiagnosticType::Scope, stmt.span)
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
                    let span = stmt.span;
                    let expr_value = e
                        .unwrap_or(Ok(Expr {
                            span,
                            expr_kind: ExprKind::Null,
                        }))
                        .unwrap_or_else(|ty| self.default_expr(ty, span));
                    self.compile_dec(ident, expr_value)?;
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
                    self.compile_expr(Expr {
                        span: stmt.span,
                        expr_kind: ExprKind::Concat(e),
                    })?;
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
                StatementKind::Case(cases, default) => {
                    let prev_block = mem::replace(&mut self.recent_block, RecentBlock::Case);
                    self.end_case_id = self.case_id + cases.len();
                    if default.is_some() {
                        self.end_case_id += 1;
                    }
                    let mut cases_iter = cases.into_iter();
                    if let Some((value, block)) = cases_iter.next() {
                        self.c.ws("if");
                        self.conditional_case_branch(value)?;
                        self.case_branch(block)?;
                    }

                    for (value, block) in cases_iter {
                        self.c.ws("else if");
                        self.conditional_case_branch(value)?;
                        self.case_branch(block)?;
                    }

                    if let Some(block) = default {
                        self.c.ws("else");
                        self.case_branch(block)?;
                    }
                    self.c.lol_case_jmp(self.case_id);
                    self.c.ws(":\n");
                    self.recent_block = prev_block;
                }
                StatementKind::Append(source, expr) => {
                    self.c.ws("lol_append(");
                    self.compile_expr(source)?;
                    self.c.comma();
                    self.compile_expr(expr)?;
                    self.c.comma();
                    self.c.span(stmt.span);
                    self.c.wc(')');
                    self.c.semi();
                }
                StatementKind::SetItem(source, item, idx) => match idx {
                    Ok(idx_e) => {
                        self.c.ws("lol_vec_set(");
                        self.compile_expr(source)?;
                        self.c.comma();
                        self.compile_expr(idx_e)?;
                        self.c.comma();
                        self.compile_expr(item)?;
                        self.c.comma();
                        self.c.span(stmt.span);
                        self.c.wc(')');
                        self.c.semi();
                    }
                    Err(is_front) => {
                        self.c.ws(if is_front {
                            "lol_vec_set_first"
                        } else {
                            "lol_vec_set_last"
                        });
                        self.c.wc('(');
                        self.compile_expr(source)?;
                        self.c.comma();
                        self.compile_expr(item)?;
                        self.c.comma();
                        self.c.span(stmt.span);
                        self.c.wc(')');
                        self.c.semi();
                    }
                },
                StatementKind::Import(..) => {}
            }
        }

        Ok(())
    }

    fn conditional_case_branch(&mut self, expr: Expr) -> Failible<()> {
        self.c.ws("(lol_to_bool(lol_eq(");
        self.c.it();
        self.c.comma();
        let span = expr.span;
        self.compile_expr(expr)?;
        self.c.comma();
        self.c.span(span);
        self.c.ws(")))");
        Ok(())
    }

    fn case_branch(&mut self, block: Block) -> Failible<()> {
        self.begin_scope();
        self.c.lol_case_jmp(self.case_id);
        self.c.ws(":\n");
        self.compile(block)?;
        self.c.ws("goto ");
        self.case_id += 1;
        self.c.lol_case_jmp(self.case_id);
        self.c.semi();
        self.end_scope();
        Ok(())
    }

    pub fn get_str(self) -> String {
        self.c.output()
    }
}
