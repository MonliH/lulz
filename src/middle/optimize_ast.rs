use crate::frontend::ast::{Block, Expr, ExprKind::*, OpTy, Statement, StatementKind::*};
use std::mem;

trait AstOptPass {
    fn apply(&mut self, ast: Block) -> Block;
}

#[inline]
fn apply_passes(passes: &mut [&mut dyn AstOptPass], block: Block) -> Block {
    let mut current = block;
    for pass in passes.iter_mut() {
        current = (*pass).apply(current);
    }
    current
}

trait ExprRewrite {
    fn rewrite(&mut self, expr: Expr) -> Expr;
}

struct RewriteExpr<T>(T)
where
    T: ExprRewrite;

struct CmpRewrite;

impl<R> RewriteExpr<R>
where
    R: ExprRewrite,
{
    fn rew(&mut self, e: Expr) -> Expr {
        self.0.rewrite(e)
    }

    fn opt(&mut self, stmt: Statement) -> Statement {
        let kind = stmt.statement_kind;
        let statement_kind = match kind {
            Append(e1, e2) => Append(self.rew(e1), self.rew(e2)),
            SetItem(e1, e2, Ok(e3)) => SetItem(self.rew(e1), self.rew(e2), Ok(self.rew(e3))),
            SetItem(e1, e2, Err(v)) => SetItem(self.rew(e1), self.rew(e2), Err(v)),
            Assignment(id, e) => Assignment(id, self.rew(e)),
            DecAssign(id, e) => DecAssign(id, e.map(|ex| ex.map(|expr| self.rew(expr)))),
            FunctionDef(id, args, bl) => FunctionDef(id, args, self.apply(bl)),
            Expr(e) => Expr(self.rew(e)),
            Case(conds, def) => Case(
                conds
                    .into_iter()
                    .map(|(e, bl)| (self.rew(e), self.apply(bl)))
                    .collect(),
                def.map(|bl| self.apply(bl)),
            ),
            If(ifb, elifs, elseb) => If(
                ifb.map(|bl| self.apply(bl)),
                elifs
                    .into_iter()
                    .map(|(e, bl)| (self.rew(e), self.apply(bl)))
                    .collect(),
                elseb.map(|bl| self.apply(bl)),
            ),
            Loop {
                block_name,
                fn_id,
                block,
            } => Loop {
                block_name,
                fn_id: fn_id.map(|(a, b, c)| (a, b, c.map(|(d, e)| (d, self.rew(e))))),
                block: self.apply(block),
            },
            Return(e) => Return(self.rew(e)),
            Print(es, ln) => Print(es.into_iter().map(|e| self.rew(e)).collect(), ln),
            Input(..) | Import(..) | MutCast(..) | Break => kind,
        };
        Statement {
            span: stmt.span,
            statement_kind,
        }
    }
}

impl<R> AstOptPass for RewriteExpr<R>
where
    R: ExprRewrite,
{
    fn apply(&mut self, ast: Block) -> Block {
        Block(
            ast.0.into_iter().map(|stmt| self.opt(stmt)).collect(),
            ast.1,
        )
    }
}

impl Block {
    pub fn opt(&mut self) {
        let owned = mem::take(self);
        *self = apply_passes(&mut [&mut RewriteExpr(CmpRewrite)], owned);
    }
}

impl ExprRewrite for CmpRewrite {
    fn rewrite(&mut self, expr: Expr) -> Expr {
        let kind = expr.expr_kind;
        let expr_kind = match kind {
            Concat(exps) => Concat(exps.into_iter().map(|e| self.rewrite(e)).collect()),
            FunctionCall(id, exprs) => {
                FunctionCall(id, exprs.into_iter().map(|e| self.rewrite(e)).collect())
            }
            Cast(e, ty) => Cast(Box::new(self.rewrite(*e)), ty),
            Operator(op, e1_box, e2_box) => {
                let e1_orig = *e1_box;
                let e2_orig = *e2_box;

                match (
                    e1_orig.expr_kind.side_effects() || e2_orig.expr_kind.side_effects(),
                    op,
                    e1_orig,
                    e2_orig,
                ) {
                    (
                        false,
                        op1,
                        Expr {
                            expr_kind: Operator(op2, cmp2, cmp3),
                            span,
                        },
                        e1,
                    ) => {
                        let new_op = match (op1, op2) {
                            (OpTy::Equal, OpTy::Max) => Some(OpTy::GTE),
                            (OpTy::Equal, OpTy::Min) => Some(OpTy::LTE),
                            (OpTy::NotEq, OpTy::Max) => Some(OpTy::LT),
                            (OpTy::NotEq, OpTy::Min) => Some(OpTy::GT),
                            _ => None,
                        };
                        match new_op {
                            Some(new_op) => {
                                if e1 == *cmp2 {
                                    Operator(new_op, Box::new(e1), cmp3)
                                } else if *cmp3 == e1 {
                                    Operator(new_op, Box::new(e1), cmp2)
                                } else {
                                    Operator(
                                        op1,
                                        Box::new(Expr {
                                            expr_kind: Operator(op2, cmp2, cmp3),
                                            span,
                                        }),
                                        Box::new(e1),
                                    )
                                }
                            }
                            None => Operator(
                                op1,
                                Box::new(Expr {
                                    expr_kind: Operator(op2, cmp2, cmp3),
                                    span,
                                }),
                                Box::new(e1),
                            ),
                        }
                    }
                    (
                        false,
                        op1,
                        e1,
                        Expr {
                            expr_kind: Operator(op2, cmp2, cmp3),
                            span,
                        },
                    ) => {
                        let new_op = match (op1, op2) {
                            (OpTy::Equal, OpTy::Max) => Some(OpTy::GTE),
                            (OpTy::Equal, OpTy::Min) => Some(OpTy::LTE),
                            (OpTy::NotEq, OpTy::Max) => Some(OpTy::LT),
                            (OpTy::NotEq, OpTy::Min) => Some(OpTy::GT),
                            _ => None,
                        };
                        match new_op {
                            Some(new_op) => {
                                if e1 == *cmp2 {
                                    Operator(new_op, Box::new(e1), cmp3)
                                } else if *cmp3 == e1 {
                                    Operator(new_op, Box::new(e1), cmp2)
                                } else {
                                    Operator(
                                        op1,
                                        Box::new(e1),
                                        Box::new(Expr {
                                            expr_kind: Operator(op2, cmp2, cmp3),
                                            span,
                                        }),
                                    )
                                }
                            }
                            None => Operator(
                                op1,
                                Box::new(e1),
                                Box::new(Expr {
                                    expr_kind: Operator(op2, cmp2, cmp3),
                                    span,
                                }),
                            ),
                        }
                    }
                    (_, op, e1, e2) => Operator(op, Box::new(e1), Box::new(e2)),
                }
            }
            UnaryOp(op, e) => UnaryOp(op, Box::new(self.rewrite(*e))),
            All(es) => All(es.into_iter().map(|e| self.rewrite(e)).collect()),
            Any(es) => Any(es.into_iter().map(|e| self.rewrite(e)).collect()),
            GetItem(e1, Ok(e2)) => {
                GetItem(Box::new(self.rewrite(*e1)), Ok(Box::new(self.rewrite(*e2))))
            }
            GetItem(e, Err(v)) => GetItem(Box::new(self.rewrite(*e)), Err(v)),
            _ => kind,
        };
        Expr {
            expr_kind,
            span: expr.span,
        }
    }
}
