use hashbrown::HashSet;

use crate::diagnostics::prelude::*;
use crate::runtime::builtins;
use crate::{diagnostics::Failible, frontend::ast::*};
use std::fmt::Write;

use super::interner::{Interner, StrId};

pub struct Translator {
    pub code: String,
    interner: Interner,
    local_scope: bool,
    globals: HashSet<StrId>,
    locals: Vec<HashSet<StrId>>,
}

type TransRes = Failible<()>;

impl Translator {
    pub fn new(interner: Interner) -> Self {
        Self {
            code: String::new(),
            interner,
            local_scope: false,
            globals: HashSet::new(),
            locals: Vec::new(),
        }
    }

    fn lparen(&mut self) {
        self.writec('(');
    }

    fn comma(&mut self) {
        self.writec(',');
    }

    fn rparen(&mut self) {
        self.writec(')');
    }

    fn quotation(&mut self) {
        self.writec('"');
    }

    fn newline(&mut self) {
        self.writec('\n');
    }

    fn space(&mut self) {
        self.writec(' ');
    }

    fn eq(&mut self) {
        self.writec('=');
    }

    fn nil(&mut self) {
        self.writes("nil");
    }

    fn boolean(&mut self, b: bool) {
        self.writes(if b { "true" } else { "false" })
    }

    fn end(&mut self) {
        self.writes("end");
    }

    fn function(&mut self) {
        self.writes("function");
    }

    fn local(&mut self) {
        self.writes("local");
    }

    fn it_var(&mut self) {
        self.writes("_lulz_it");
    }

    fn list(&mut self, first: Option<&Expr>, items: &[Expr]) -> TransRes {
        let mut items = items.iter();
        let next = first.map(|f| Some(f)).unwrap_or_else(|| items.next());
        if let Some(first) = next {
            self.expr(first)?;
            for item in items {
                self.comma();
                self.expr(item)?;
            }
        }
        Ok(())
    }

    fn list_ref(&mut self, first: Option<&Expr>, items: &[&Expr]) -> TransRes {
        let mut items = items.iter();
        let next = first
            .map(|f| Some(f))
            .unwrap_or_else(|| items.next().copied());
        if let Some(first) = next {
            self.expr(first)?;
            for item in items {
                self.comma();
                self.expr(item)?;
            }
        }
        Ok(())
    }

    fn array(&mut self, first: Option<&Expr>, items: &[Expr]) -> TransRes {
        self.writec('{');
        self.list(first, items)?;
        self.writec('}');
        Ok(())
    }

    fn writes(&mut self, s: &str) {
        // Ignore, as this will probably not error
        let _ = self.code.write_str(s);
    }

    fn writec(&mut self, c: char) {
        let _ = self.code.write_char(c);
    }

    fn _block(&mut self, block: Block) -> TransRes {
        for stmt in block.0.into_iter() {
            self.stmt(stmt)?;
            self.newline()
        }
        Ok(())
    }

    pub fn outer_block(&mut self, block: Block) -> TransRes {
        self._block(block)
    }

    fn block(&mut self, block: Block) -> TransRes {
        let prev = std::mem::replace(&mut self.local_scope, true);
        self.locals.push(HashSet::new());
        self._block(block)?;
        self.locals.pop();
        self.local_scope = prev;
        Ok(())
    }

    fn stmt(&mut self, stmt: Stmt) -> TransRes {
        match stmt.ty {
            StmtTy::Print(exprs, no_newline) => self.call(
                if no_newline {
                    builtins::io::LUA_PRINT
                } else {
                    builtins::io::LUA_PRINTLN
                },
                None,
                &exprs,
            )?,
            StmtTy::DecAssign(ref name, expr) => match expr {
                Some(var) => {
                    let e = match var {
                        Ok(e) => e,
                        Err(t) => Expr {
                            ty: t.default_expr_kind(),
                            span: stmt.span,
                        },
                    };
                    self.declaration(&name, &e)?;
                }
                None => self.declaration(
                    &name,
                    &Expr {
                        ty: ExprTy::Null,
                        span: Span::default(),
                    },
                )?,
            },
            StmtTy::Assignment(name, expr) => {
                self.assignment(&name, &expr)?;
            }
            StmtTy::FunctionDef(fn_name, args, block) => {
                self.function();
                self.space();
                self.ident(&fn_name);
                self.lparen();
                let mut args = args.iter();
                if let Some(first) = args.next() {
                    self.ident(first);
                    self.comma();
                    for arg in args {
                        self.ident(arg)
                    }
                }
                self.rparen();
                self.block(block)?;
                self.end();
            }
            StmtTy::Expr(expr) => {
                self.it_var();
                self.eq();
                self.expr(&expr)?;
            }
            _ => todo!("Statement not implemented: {:?}", stmt),
        }
        Ok(())
    }

    fn declaration(&mut self, name: &Ident, expr: &Expr) -> TransRes {
        if self.is_in_current_scope(name) {
            // Don't allow declaration in the same scope
            return Err(Diagnostic::build(DiagnosticType::Scope, name.1)
                .annotation(
                    Cow::Owned(format!(
                        "variable `{}` cannot be re-declared",
                        self.id_to_str(name)
                    )),
                    name.1,
                )
                .note(Cow::Borrowed(
                    "declarations of the same name can only occur in different scopes",
                ))
                .into());
        }

        if self.local_scope {
            self.define_local(name);
            self.local();
            self.space();
        } else {
            self.globals.insert(name.0);
        }

        self.ident(name);
        self.eq();
        self.expr(expr)?;

        Ok(())
    }

    fn define_local(&mut self, name: &Ident) {
        self.locals.last_mut().unwrap().insert(name.0);
    }

    fn undefined_var_error(&self, name: &Ident) -> Diagnostic {
        Diagnostic::build(DiagnosticType::UnknownSymbol, name.1).annotation(
            Cow::Owned(format!(
                "variable `{}` does not exist in this scope",
                self.id_to_str(name)
            )),
            name.1,
        )
    }

    fn assignment(&mut self, name: &Ident, expr: &Expr) -> TransRes {
        if self.is_defined(name) {
            self.ident(name);
            self.eq();
            self.expr(expr)?;
            return Ok(());
        }
        Err(self.undefined_var_error(name).into())
    }

    fn id_to_str<'a>(&'a self, ident: &Ident) -> &'a str {
        self.interner.lookup(ident.0)
    }

    fn is_global(&self, name: &Ident) -> bool {
        self.globals.contains(&name.0)
    }

    fn is_local(&self, name: &Ident) -> bool {
        self.locals.iter().any(|scope| scope.contains(&name.0))
    }

    fn is_defined(&self, name: &Ident) -> bool {
        self.is_global(name) || self.is_local(name)
    }

    fn is_in_current_scope(&mut self, name: &Ident) -> bool {
        if self.local_scope {
            self.is_local(name)
        } else {
            self.is_global(name)
        }
    }

    fn call(&mut self, f: &str, first: Option<&Expr>, args: &[Expr]) -> TransRes {
        self.writes(f);
        self.lparen();
        self.list(first, args)?;
        self.rparen();
        Ok(())
    }

    fn call_ref(&mut self, f: &str, first: Option<&Expr>, args: &[&Expr]) -> TransRes {
        self.writes(f);
        self.lparen();
        self.list_ref(first, args)?;
        self.rparen();
        Ok(())
    }

    fn ident(&mut self, id: &Ident) {
        self.writes(&format!("_{}", id.0.inner()));
    }

    fn raw_string(&mut self, s: &str) {
        self.quotation();
        self.writes(&s.replace("\\", "\\\\").replace("\"", "\\\""));
        self.quotation();
    }

    fn expr(&mut self, expr: &Expr) -> TransRes {
        match &expr.ty {
            ExprTy::Int(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::Float(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::String(s) => self.raw_string(s),
            ExprTy::Null => {
                self.nil();
            }
            ExprTy::Bool(b) => {
                self.boolean(*b);
            }
            ExprTy::Variable(ref id) => {
                if !self.is_defined(id) {
                    return Err(self.undefined_var_error(id).into());
                }
                self.ident(id);
            }
            ExprTy::Operator(op_ty, ref l, ref r) => self.operator(*op_ty, &*l, &*r)?,
            ExprTy::Span(span) => {
                self.write_span(*span);
            }
            ExprTy::FunctionCall(fn_name, args) => {
                self.expr(&Self::make_id_expr(*fn_name))?;
                self.lparen();
                self.list(None, &args)?;
                self.rparen();
            }
            _ => todo!("Expression not implemented: {:?}", expr),
        }
        Ok(())
    }

    fn make_id_expr(id: Ident) -> Expr {
        Expr {
            span: id.1,
            ty: ExprTy::Variable(id),
        }
    }

    fn write_span(&mut self, span: Span) {
        self.array(
            None,
            &[
                Self::int_expr(span.s as i64),
                Self::int_expr(span.e as i64),
                Self::int_expr(span.file as i64),
            ],
        )
        .unwrap();
    }

    fn int_expr(n: i64) -> Expr {
        Expr {
            ty: ExprTy::Int(n),
            span: Span::default(),
        }
    }

    fn span_expr(span: Span) -> Expr {
        Expr {
            ty: ExprTy::Span(span),
            span,
        }
    }

    fn operator(&mut self, op_ty: OpTy, l: &Expr, r: &Expr) -> TransRes {
        let span = l.span.combine(&r.span);
        let span_expr = &Self::span_expr(span);
        match op_ty {
            OpTy::Add => self.call_ref(builtins::ops::LUA_ADD, None, &[span_expr, l, r])?,
            OpTy::Sub => self.call_ref(builtins::ops::LUA_SUB, None, &[span_expr, l, r])?,
            OpTy::Mul => self.call_ref(builtins::ops::LUA_MUL, None, &[span_expr, l, r])?,
            OpTy::Div => self.call_ref(builtins::ops::LUA_DIV, None, &[span_expr, l, r])?,
            OpTy::Mod => self.call_ref(builtins::ops::LUA_MOD, None, &[span_expr, l, r])?,

            OpTy::And => self.call_ref(builtins::ops::LUA_AND, None, &[span_expr, l, r])?,
            OpTy::Or => self.call_ref(builtins::ops::LUA_OR, None, &[span_expr, l, r])?,

            OpTy::Equal => self.call_ref(builtins::ops::LUA_EQ, None, &[span_expr, l, r])?,
            OpTy::NotEq => self.call_ref(builtins::ops::LUA_NEQ, None, &[span_expr, l, r])?,

            OpTy::GT => self.call_ref(builtins::ops::LUA_GT, None, &[span_expr, l, r])?,
            OpTy::LT => self.call_ref(builtins::ops::LUA_LT, None, &[span_expr, l, r])?,
            OpTy::GTE => self.call_ref(builtins::ops::LUA_GTE, None, &[span_expr, l, r])?,
            OpTy::LTE => self.call_ref(builtins::ops::LUA_LTE, None, &[span_expr, l, r])?,
            op => todo!("Operator not implemented: {:?}", op),
        }
        Ok(())
    }
}
