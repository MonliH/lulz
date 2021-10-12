//! Lazy code objects, as described in "Building JIT
//! compilers for dynamic languages with low development effort":
//!
//! Saleil, B., & Feeley, M. (2018, November). Building JIT
//! compilers for dynamic languages with low development effort.
//! In Proceedings of the 10th ACM SIGPLAN International Workshop
//! on Virtual Machines and Intermediate Languages (pp. 36-46).

use lightning_sys::JitState;
use rustc_hash::FxHashMap;

use crate::frontend::ast::LolTy;

use super::interner::StrId;

pub type LcoRef<'a> = Option<&'a Lco<'a>>;

#[derive(Clone, Copy, Debug)]
pub enum Lco<'a> {
    Float(f32, LcoRef<'a>),
    Int(i64, LcoRef<'a>),
    Bool(bool, LcoRef<'a>),
    Null(LcoRef<'a>),
    Variable(StrId, LcoRef<'a>),

    NewVariable(StrId, LcoRef<'a>),
    Print(LcoRef<'a>),

    HasType {
        ty: LolTy,
        yes: LcoRef<'a>,
        no: LcoRef<'a>,
    },
}

pub struct CompilationCtx {
    stack: Vec<ValueInfo>,
    env: FxHashMap<StrId, ValueInfo>,
}

#[derive(Clone, Copy, Debug)]
pub enum ValueInfo {
    ConstBool(bool),
    ConstFloat(f32),
    ConstInt(i64),
    ConstNull,

    Type(LolTy),
}
