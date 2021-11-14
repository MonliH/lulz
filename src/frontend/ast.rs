use std::hash::{Hash, Hasher};

use crate::{backend::interner::StrId, diagnostics::Span};

#[derive(Debug, Clone)]
pub struct Ident(pub StrId, pub Span);

impl std::cmp::PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Block(pub Vec<Stmt>, pub Span);

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub ty: StmtTy,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LoopCond {
    Forever,
    Till(Expr),
    While(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtTy {
    Assignment(Ident, Expr),
    DecAssign(Ident, Option<Result<Expr, LolTy>>),
    Import(Ident),
    FunctionDef(Ident, Vec<Ident>, Block),
    Expr(Expr),
    Case(Vec<(Expr, Block)>, Option<Block>),
    If(Option<Block>, Vec<(Expr, Block)>, Option<Block>),
    MutCast(Ident, LolTy),
    Break,
    Loop {
        block_name: Ident,
        fn_id: Option<(
            // Function name
            Ident,
            // Variable name
            Ident,
            LoopCond,
        )>,
        block: Block,
    },
    Return(Expr),
    // bool = no_newline
    Print(Vec<Expr>, bool),
    Input(Ident),

    /// e1.append(e2)
    /// Append(source collection, item)
    Append(Expr, Expr),
    // index item:
    // bool:  true = FRONT, false = BACK
    // Ident: index is the ident
    /// SetItem(source, item, index)
    SetItem(Expr, Expr, Result<Expr, bool>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub ty: ExprTy,
    pub span: Span,
}

impl std::cmp::PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

#[derive(Debug, Clone, Eq)]
pub struct InterpEntry(pub usize, pub String, pub Span);

impl std::cmp::PartialEq for InterpEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExprTy {
    Span(Span),
    Float(f64),
    Int(i64),
    String(String),
    InterpStr(String, Vec<InterpEntry>),
    Bool(bool),
    List(Vec<Expr>),

    Null,
    It,

    Variable(Ident),
    FunctionCall(Ident, Vec<Expr>),
    Concat(Vec<Expr>),
    Cast(Box<Expr>, LolTy),

    Operator(OpTy, Box<Expr>, Box<Expr>),

    All(Vec<Expr>),
    Any(Vec<Expr>),
    UnaryOp(UnOpTy, Box<Expr>),

    /// GetItem(source, index)
    GetItem(Box<Expr>, Result<Box<Expr>, bool>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum UnOpTy {
    Not,
    Length,
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
    Funkshun,
    Lizt,
}

impl LolTy {
    pub fn as_cast(&self) -> &'static str {
        match self {
            LolTy::Noob => "noob",
            LolTy::Yarn => "yarn",
            LolTy::Troof => "troof",
            LolTy::Numbar => "numbar",
            LolTy::Numbr => "numbr",
            LolTy::Funkshun => "funkshun",
            LolTy::Lizt => "lizt",
        }
    }

    pub fn default_expr_kind(&self) -> ExprTy {
        match self {
            LolTy::Troof => ExprTy::Bool(false),
            LolTy::Numbar => ExprTy::Float(0.0),
            LolTy::Numbr => ExprTy::Int(0),
            LolTy::Yarn => ExprTy::String("".to_string()),
            LolTy::Lizt => ExprTy::List(Vec::new()),
            LolTy::Funkshun | LolTy::Noob => ExprTy::Null,
        }
    }
}
