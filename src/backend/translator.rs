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
            _ => {}
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
            _ => {}
        }
        Ok(())
    }
}
