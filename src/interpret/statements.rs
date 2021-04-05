use super::{expressions::Eval, helpers::*};
use crate::{ast::*, diagnostics::Failible};

pub trait Exec {
    fn exec(&self, ctx: &mut Ctx) -> Failible<()>;
}

impl Exec for Block {
    fn exec(&self, ctx: &mut Ctx) -> Failible<()> {
        for stmt in &self.0 {
            stmt.exec(ctx)?;
        }
        Ok(())
    }
}

impl Exec for Statement {
    fn exec(&self, ctx: &mut Ctx) -> Failible<()> {
        match &self.statement_kind {
            StatementKind::Print(expr, no_newline) => {
                print!("{}", expr.eval(ctx)?.cast(true, Type::Str, self.span)?);
                if !no_newline {
                    println!()
                }
            }
            _ => (),
        }
        Ok(())
    }
}
