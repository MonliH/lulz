use smol_str::SmolStr;
use std::ops::{Deref, DerefMut};

use crate::diagnostics::Span;

pub struct Ident(pub SmolStr, pub Span);

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

pub struct Block(pub Vec<Statement>, pub Span);

pub struct Statement {
    pub statement_kind: StatementKind,
    pub span: Span,
}

pub enum StatementKind {
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
        /// Bool represents `till` or `wile`
        /// true = `till`
        /// false = `wile`
        pred: Option<(bool, Expr)>,
        block: Block,
    },
    Return(Expr),
    Print(Expr),
}

pub struct Expr {
    pub expr_kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Float(f64),
    Int(i64),
    String(String),
    Boolean(bool),
    Variable(Ident),
    FunctionCall(Ident, Vec<Expr>),
}
