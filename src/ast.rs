use smol_str::SmolStr;
use std::ops::{Deref, DerefMut};

use crate::diagnostics::Span;

struct Ident(pub SmolStr, Span);

impl Deref for Ident {
    type Target = SmolStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ident {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct Block(Vec<Statement>, Span);

struct Statement {
    statement_kind: StatementKind,
    span: Span,
}

enum StatementKind {
    Assignment(Ident, Expr),
    DecAssign(Ident, Expr),
    Import(Ident),
    FunctionDef(Ident, Vec<Ident>, Block),
    Expr(Expr),
    Case(Vec<(Expr, Block)>, Option<Block>),
    If(Option<Block>, Vec<(Expr, Block)>, Option<Block>),
    Break,
    Loop {
        block_name: Ident,
        func: Ident,
        index: Ident,
        pred: Expr,
        till: bool,
        block: Block,
    },
    Return(Expr),
    Print(Expr),
}

struct Expr {
    expr_kind: ExprKind,
    span: Span,
}

enum ExprKind {
    Float(f64),
    Int(i64),
    String(String),
    Boolean(bool),
    Variable(Ident),
    FunctionCall(Ident, Vec<Expr>),
}
