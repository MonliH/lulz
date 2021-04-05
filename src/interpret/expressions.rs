use super::helpers::*;
use crate::{ast::*, diagnostics::Failible};

pub trait Eval {
    fn eval(&self, ctx: &mut Ctx) -> Failible<Prim>;
}

impl Eval for Expr {
    fn eval(&self, ctx: &mut Ctx) -> Failible<Prim> {
        Ok(Prim::Int(10))
    }
}
