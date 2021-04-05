use super::{helpers::*, statements::run_function};
use crate::{ast::*, diagnostics::prelude::*};

pub trait Eval {
    fn eval(&self, ctx: &mut Ctx) -> Failible<Prim>;
}

impl Eval for Expr {
    fn eval(&self, ctx: &mut Ctx) -> Failible<Prim> {
        Ok(match self.expr_kind {
            ExprKind::Float(f) => Prim::Float(f),
            ExprKind::Int(i) => Prim::Int(i),
            ExprKind::String(ref s) => Prim::String(s.to_string()),
            ExprKind::Variable(ref v) => ctx.lookup(v)?.clone(),
            ExprKind::Bool(b) => Prim::Bool(b),
            ExprKind::Concat(ref es) => Prim::String(
                es.iter()
                    .map(|e| Ok(e.eval(ctx)?.cast(true, Type::Str, e.span)?.unwrap_str()))
                    .collect::<Failible<Vec<_>>>()?
                    .join(""),
            ),
            ExprKind::Cast(ref expr, ty) => expr.eval(ctx)?.cast(false, ty, self.span)?,
            ExprKind::FunctionCall(ref fn_name, ref es) => {
                self.eval_function_call(ctx, fn_name, &es)?
            }
        })
    }
}

impl Expr {
    fn eval_function_call(&self, ctx: &mut Ctx, name: &Ident, es: &[Expr]) -> Failible<Prim> {
        let exprs = es
            .into_iter()
            .map(|e| e.eval(ctx))
            .collect::<Failible<Vec<_>>>()?;
        let func = ctx.lookup(name)?;
        match func {
            Prim::Function(args, code) => run_function(ctx.clone(), exprs, args, code, self.span),
            _ => Err(
                Diagnostic::build(Level::Error, DiagnosticType::Type, name.1)
                    .annotation(
                        Cow::Owned(format!("`{}` is not a {}", name.0, Type::Function)),
                        name.1,
                    )
                    .into(),
            ),
        }
    }
}
