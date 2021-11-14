use crate::diagnostics::Span;
use crate::runtime::builtins;
use crate::{diagnostics::Failible, frontend::ast::*};
use std::fmt::Write;

use super::interner::Interner;

pub struct Translator {
    pub code: String,
    interner: Interner
}

type TransRes = Failible<()>;

impl Translator {
    pub fn new(interner: Interner) -> Self {
        Self {
            code: String::new(),
            interner
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

    fn eq(&mut self) {
        self.writec('=');
    }

    fn nil(&mut self) {
        self.call_ref(builtins::null::LUA_NEW_NULL, None, &[])
            .unwrap();
    }

    fn boolean(&mut self, b: bool) {
        self.writes(if b { "true" } else { "false" })
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

    pub fn block(&mut self, block: Block) -> TransRes {
        for stmt in block.0.into_iter() {
            self.stmt(stmt)?;
            self.newline()
        }
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
                    self.assignment(&name, &e)?;
                }
                None => self.assignment(
                    &name,
                    &Expr {
                        ty: ExprTy::Null,
                        span: Span::default(),
                    },
                )?,
            },
            StmtTy::Assignment(name, expr) => {
                // TODO: check if init'ed
                self.assignment(&name, &expr)?;
            }
            _ => todo!("Statement not implemented: {:?}", stmt),
        }
        Ok(())
    }

    fn assignment(&mut self, name: &Ident, expr: &Expr) -> TransRes {
        self.ident(name);
        self.eq();
        self.expr(expr)?;
        Ok(())
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
        match expr.ty {
            ExprTy::Int(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::Float(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::String(ref s) => {self.raw_string(s)}
            ExprTy::Null => {
                self.nil();
            }
            ExprTy::Bool(b) => {
                self.boolean(b);
            }
            ExprTy::Variable(ref id) => {
                self.writes(builtins::null::LUA_CHECK_VARIABLE);
                self.lparen();
                self.write_span(expr.span);
                self.comma();
                let var = self.interner.lookup(id.0).to_string();
                self.raw_string(&var);
                self.comma();
                self.ident(id);
                self.rparen();
            }
            ExprTy::Operator(op_ty, ref l, ref r) => self.operator(op_ty, &*l, &*r)?,
            ExprTy::Span(span) => {
                self.write_span(span);
            }
            _ => todo!("Expression not implemented: {:?}", expr),
        }
        Ok(())
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
