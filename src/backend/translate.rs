use toolshed::Arena;

use super::lco::{Lco, LcoRef};
use crate::frontend::ast::{ExprKind, Statement, StatementKind};

macro_rules! recurse {
    ($ast: expr, $arena: expr) => {
        translate_ast(&$ast[1..], $arena)
    };
}

pub fn translate_ast<'a>(ast: &[Statement], arena: &'a Arena) -> LcoRef<'a> {
    if ast.is_empty() {
        return None;
    }
    let stmt = &ast[0];
    Some(match &stmt.statement_kind {
        StatementKind::Expr(e) => match e.expr_kind {
            ExprKind::Float(f) => arena.alloc(Lco::Float(f, recurse!(ast, arena))),
            ExprKind::Int(i) => arena.alloc(Lco::Int(i, recurse!(ast, arena))),
            ExprKind::Bool(b) => arena.alloc(Lco::Bool(b, recurse!(ast, arena))),
            ExprKind::Null => arena.alloc(Lco::Null(recurse!(ast, arena))),
            _ => panic!("Unimplemented expression kind"),
        },
        _ => panic!("Unimplemented statement kind"),
    })
}
