use smol_str::SmolStr;
use std::{
    hash::{Hash, Hasher},
    intrinsics::transmute,
    ops::{Deref, DerefMut},
};

use crate::diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Ident(pub SmolStr, pub Span);
impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Block(pub Vec<Statement>, pub Span);

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub statement_kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind {
    Assignment(Ident, Expr),
    DecAssign(Ident, Option<Expr>),
    Import(Ident),
    FunctionDef(Ident, Vec<Ident>, Block),
    Expr(Expr),
    Case(Vec<(Expr, Block)>, Option<Block>),
    If(Option<Block>, Vec<(Expr, Block)>, Option<Block>),
    MutCast(Ident, Type),
    Break,
    Loop {
        block_name: Ident,
        func: Option<Ident>,
        index: Option<Ident>,
        /// Bool represents `till` or `wile`
        /// true = `till`
        /// false = `wile`
        pred: Option<(bool, Expr)>,
        block: Block,
    },
    Return(Expr),
    Print(Expr, bool),
    Input(Ident),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub expr_kind: ExprKind,
    pub span: Span,
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.expr_kind == other.expr_kind
    }
}

#[derive(Debug, Clone, Eq)]
pub struct InterpEntry(pub usize, pub String, pub Span);
impl PartialEq for InterpEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExprKind {
    Float(f64),
    Int(i64),
    String(String),
    InterpStr(String, Vec<InterpEntry>),
    Bool(bool),

    Null,

    Variable(Ident),
    FunctionCall(Ident, Vec<Expr>),
    Concat(Vec<Expr>),
    Cast(Box<Expr>, Type),

    Operator(OpTy, Box<Expr>, Box<Expr>),

    All(Vec<Expr>),
    Any(Vec<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpTy {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Min,
    Max,

    And,
    Or,
    Xor,

    Equal,
    NotEq,

    GT,
    LT,
    GTE,
    LTE,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Null = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    Str = 4,
}

impl Type {
    pub fn from_num(id: u8) -> Option<Self> {
        if id <= 4 {
            Some(unsafe { transmute(id) })
        } else {
            None
        }
    }
}
