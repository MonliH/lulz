use super::builtins;
use crate::{diagnostics::Failible, frontend::ast::*};
use std::fmt::Write;

pub struct Translator {
    pub code: String,
}

type TransRes = Failible<()>;

impl Translator {
    pub fn new() -> Self {
        Self {
            code: String::new(),
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
        self.writes("nil");
    }

    fn boolean(&mut self, b: bool) {
        self.writes(if b { "true" } else { "false" })
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
                &exprs,
            )?,
            StmtTy::DecAssign(ref name, expr) => {
                if let Some(var) = expr {
                    let e = match var {
                        Ok(e) => e,
                        Err(t) => Expr {
                            ty: t.default_expr_kind(),
                            span: stmt.span,
                        },
                    };
                    self.assignment(&name, &e)?;
                }
            }
            StmtTy::Assignment(name, expr) => {
                // TODO: check if init'ed
                self.assignment(&name, &expr)?;
            }
            _ => todo!("Statement not implemented: {:?}", stmt)
        }
        Ok(())
    }

    fn assignment(&mut self, name: &Ident, expr: &Expr) -> TransRes {
        self.ident(name);
        self.eq();
        self.expr(expr)?;
        Ok(())
    }

    fn call(&mut self, f: &str, args: &[Expr]) -> TransRes {
        self.writes(f);
        self.lparen();
        let mut args = args.iter();
        if let Some(first) = args.next() {
            self.expr(first)?;
            for arg in args {
                self.comma();
                self.expr(arg)?;
            }
        }
        self.rparen();
        Ok(())
    }

    fn call_ref(&mut self, f: &str, args: &[&Expr]) -> TransRes {
        self.writes(f);
        self.lparen();
        let mut args = args.iter();
        if let Some(first) = args.next() {
            self.expr(first)?;
            for arg in args {
                self.comma();
                self.expr(arg)?;
            }
        }
        self.rparen();
        Ok(())
    }

    fn ident(&mut self, id: &Ident) {
        self.writes(&format!("_{}", id.0.inner()));
    }

    fn expr(&mut self, expr: &Expr) -> TransRes {
        match expr.ty {
            ExprTy::Int(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::Float(i) => {
                self.writes(&i.to_string());
            }
            ExprTy::String(ref s) => {
                self.quotation();
                self.writes(&s.replace("\\", "\\\\").replace("\"", "\\\""));
                self.quotation();
            }
            ExprTy::Null => {
                self.nil();
            }
            ExprTy::Bool(b) => {
                self.boolean(b);
            }
            ExprTy::Variable(ref id) => self.ident(id),
            ExprTy::Operator(op_ty, ref l, ref r) => self.operator(op_ty, &*l, &*r)?,
            _ => todo!("Expression not implemented: {:?}", expr)
        }
        Ok(())
    }

    fn operator(&mut self, op_ty: OpTy, l: &Expr, r: &Expr) -> TransRes {
        match op_ty {
            OpTy::Add => self.call_ref(builtins::ops::LUA_ADD, &[l, r])?,
            OpTy::Sub => self.call_ref(builtins::ops::LUA_SUB, &[l, r])?,
            OpTy::Mul => self.call_ref(builtins::ops::LUA_MUL, &[l, r])?,
            OpTy::Div => self.call_ref(builtins::ops::LUA_DIV, &[l, r])?,
            OpTy::Mod => self.call_ref(builtins::ops::LUA_MOD, &[l, r])?,

            OpTy::And => self.call_ref(builtins::ops::LUA_AND, &[l, r])?,
            OpTy::Or => self.call_ref(builtins::ops::LUA_OR, &[l, r])?,

            OpTy::Equal => self.call_ref(builtins::ops::LUA_EQ, &[l, r])?,
            OpTy::NotEq => self.call_ref(builtins::ops::LUA_NEQ, &[l, r])?,

            OpTy::GT => self.call_ref(builtins::ops::LUA_GT, &[l, r])?,
            OpTy::LT => self.call_ref(builtins::ops::LUA_LT, &[l, r])?,
            OpTy::GTE => self.call_ref(builtins::ops::LUA_GTE, &[l, r])?,
            OpTy::LTE => self.call_ref(builtins::ops::LUA_LTE, &[l, r])?,
            op => todo!("Operator not implemented: {:?}", op)
        }
        Ok(())
    }
}
