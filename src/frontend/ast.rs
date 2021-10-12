use smol_str::SmolStr;
use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::diagnostics::Span;

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq)]
pub struct Ident(pub SmolStr, #[derivative(PartialEq = "ignore")] pub Span);

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
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
    MutCast(Ident, LolTy),
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

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq)]
pub struct Expr {
    pub expr_kind: ExprKind,
    #[derivative(PartialEq = "ignore")]
    pub span: Span,
}

#[derive(Derivative, Debug, Clone, Eq)]
#[derivative(PartialEq)]
pub struct InterpEntry(
    pub usize,
    pub String,
    #[derivative(PartialEq = "ignore")] pub Span,
);

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq)]
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
    Cast(Box<Expr>, LolTy),

    Operator(OpTy, Box<Expr>, Box<Expr>),

    All(Vec<Expr>),
    Any(Vec<Expr>),
    Not(Box<Expr>),
}

impl ExprKind {
    /// Check if an expression has side effects, currently very conservative
    pub fn side_effects(&self) -> bool {
        match self {
            Self::Float(..)
            | Self::Int(..)
            | Self::String(..)
            | Self::InterpStr(..)
            | Self::Bool(..)
            | Self::Null
            | Self::Variable(..) => false,
            Self::FunctionCall(..) => true,
            Self::Concat(es) | Self::All(es) | Self::Any(es) => {
                es.iter().any(|e| e.expr_kind.side_effects())
            }
            Self::Cast(e, _) | Self::Not(e) => e.expr_kind.side_effects(),
            Self::Operator(_, e1, e2) => {
                e1.expr_kind.side_effects() || e2.expr_kind.side_effects()
            }
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LolTy {
    Troof,
    Noob,
    Yarn,
    Numbar,
    Numbr,
    Func,
}

impl LolTy {
    pub fn as_cast(&self) -> &'static str {
        match self {
            LolTy::Noob => "noob",
            LolTy::Yarn => "yarn",
            LolTy::Troof => "troof",
            LolTy::Numbar => "numbar",
            LolTy::Numbr => "numbr",
            LolTy::Func => "fn",
        }
    }
    pub fn as_macro(&self) -> &'static str {
        match self {
            LolTy::Noob => "NULL",
            LolTy::Troof => "BOOL",
            LolTy::Numbar => "DOUBLE",
            LolTy::Yarn => "STR",
            LolTy::Numbr => "INT",
            LolTy::Func => "FUN",
        }
    }
}
