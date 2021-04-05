use std::{io, io::BufRead};

use super::{expressions::Eval, helpers::*};
use crate::{ast::*, diagnostics::prelude::*};

pub trait Exec {
    fn exec(&self, ctx: &mut Ctx) -> Failible<Option<Prim>>;
}

impl Exec for Block {
    fn exec(&self, ctx: &mut Ctx) -> Failible<Option<Prim>> {
        for stmt in &self.0 {
            if let Some(return_val) = stmt.exec(ctx)? {
                return Ok(Some(return_val));
            }
        }
        Ok(None)
    }
}

impl Exec for Statement {
    fn exec(&self, ctx: &mut Ctx) -> Failible<Option<Prim>> {
        match &self.statement_kind {
            StatementKind::Print(expr, no_newline) => {
                print!("{}", expr.eval(ctx)?.cast(true, Type::Str, self.span)?);
                if !no_newline {
                    println!()
                }
            }
            StatementKind::Input(ident) => {
                ctx.lookup(ident)?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line).unwrap();
                ctx.insert(ident.clone(), Prim::String(line));
            }
            StatementKind::Assignment(ident, expr) => {
                ctx.lookup(ident)?;
                let evaled = expr.eval(ctx)?;
                ctx.insert(ident.clone(), evaled);
            }
            StatementKind::DecAssign(ident, None) => {
                ctx.insert(ident.clone(), Prim::Null);
            }
            StatementKind::DecAssign(ident, Some(expr)) => {
                let evaled = expr.eval(ctx)?;
                ctx.insert(ident.clone(), evaled);
            }
            StatementKind::FunctionDef(name, args, block) => {
                ctx.insert(name.clone(), Prim::Function(args.clone(), block.clone()));
            }
            StatementKind::Expr(expr) => {
                let evaled = expr.eval(ctx)?;
                ctx.it = evaled;
            }
            StatementKind::Break => {
                return Ok(Some(Prim::Null));
            }
            StatementKind::Return(expr) => {
                let evaled = expr.eval(ctx)?;
                return Ok(Some(evaled));
            }
            StatementKind::Loop {
                func,
                index,
                pred,
                block,
                ..
            } => {}
            _ => todo!(),
        }
        Ok(None)
    }
}

pub fn run_function(
    mut ctx: Ctx,
    exs: Vec<Prim>,
    args: &[Ident],
    code: &Block,
    span: Span,
) -> Failible<Prim> {
    if exs.len() != args.len() {
        let mut diagnostic =
            Diagnostic::build(Level::Error, DiagnosticType::FunctionArgumentMismatch, span)
                .annotation(
                    Cow::Owned(format!(
                        "but you passed in {} argument{}",
                        exs.len(),
                        plural(exs.len())
                    )),
                    span,
                );
        if !args.is_empty() {
            let a = &args[0..(args.len() - 1)];
            diagnostic
                .annotations
                .extend(a.iter().map(|id| Annotation::new(Cow::Borrowed(""), id.1)));
            diagnostic.annotations.push(Annotation::new(
                Cow::Owned(format!(
                    "this function takes {} argument{}",
                    args.len(),
                    plural(args.len())
                )),
                args.last().unwrap().1,
            ));
        }
        return Err(diagnostic.into());
    }
    ctx.sym_tab.reserve(args.len());
    for (name, expr) in args.iter().zip(exs) {
        ctx.insert(name.clone(), expr);
    }
    let res = code.exec(&mut ctx)?;
    match res {
        Some(prim) => Ok(prim),
        None => Ok(ctx.it),
    }
}
