use crate::ast::*;
use crate::backend::interner::{Interner, StrId};
use crate::diagnostics::prelude::*;
use crate::lex::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_span: Span,
    source_id: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            current_span: Span::new(0, 0, lexer.source_id),
            source_id: lexer.source_id,
            lexer,
        }
    }

    fn next_token(&mut self) -> Failible<Token> {
        let tok = self.lexer.next()?;
        self.current_span = tok.span;
        Ok(tok)
    }

    fn peek_token(&mut self) -> Failible<&Token> {
        let raw_self = self as *mut Self;

        // SAFTEY: The "aliased" mutable borrows are sound because we drop one before using another
        // one.
        //
        // In the case of the Err(_) pattern where we "alias", the reference is dropped, so no
        // aliasing is occurring.
        //
        // This is perfectly sound and should be picked up by the compiler, see:
        // https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
        //
        // NOTE: This code actually does compile with the nightly -Zpolonius flag.
        match unsafe { raw_self.as_mut() }.unwrap().lexer.peek() {
            Ok(t) => Ok(t),
            // The original err value **is DROPPED**
            Err(_) => Err(unsafe { raw_self.as_mut() }
                .unwrap()
                .next_token()
                .unwrap_err()),
        }
    }

    fn expect(&mut self, token: TokenKind) -> Failible<Token> {
        let next = self.next_token()?;
        if std::mem::discriminant(&token) == std::mem::discriminant(&next.token_kind) {
            Ok(next)
        } else {
            Err(Diagnostic::build(DiagnosticType::Syntax, next.span)
                .annotation(
                    Cow::Owned(format!("expected {}, but found {}", token, next.token_kind)),
                    next.span,
                )
                .into())
        }
    }

    pub fn eat_lines(&mut self) -> Failible<()> {
        while self.check(&TokenKind::Break)? {}
        Ok(())
    }

    pub fn expect_lines(&mut self) -> Failible<()> {
        self.expect(TokenKind::Break)?;
        while self.check(&TokenKind::Break)? {}
        Ok(())
    }

    pub fn parse(&mut self) -> Failible<Block> {
        self.eat_lines()?;
        self.expect(TokenKind::Hai)?;
        self.version()?;
        self.expect_lines()?;
        let block = self.block(Some(&[TokenKind::Kthxbye]))?;
        self.eat_lines()?;
        self.expect(TokenKind::Kthxbye)?;
        self.eat_lines()?;
        Ok(block)
    }

    fn version(&mut self) -> Failible<()> {
        self.expect(TokenKind::Number(String::new()))?;
        self.expect(TokenKind::Dot)?;
        self.expect(TokenKind::Number(String::new()))?;
        Ok(())
    }

    #[inline]
    fn next_tok_is(&mut self, tokens_after: Option<&'static [TokenKind]>) -> Failible<bool> {
        if let Some(tokens) = tokens_after {
            for tok in tokens {
                if tok.eq(&self.peek_token()?.token_kind) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn block(&mut self, tokens_after: Option<&'static [TokenKind]>) -> Failible<Block> {
        let start_span = self.current_span.s;
        let statements = if self.next_tok_is(tokens_after)? {
            vec![]
        } else {
            let mut statements = vec![self.statement()?];
            loop {
                if self.check(&TokenKind::Break)? {
                    self.eat_lines()?;
                    if self.next_tok_is(tokens_after)? {
                        break;
                    }
                    statements.push(self.statement()?);
                } else {
                    break;
                }
            }
            statements
        };
        let end_span = self.current_span.e;

        Ok(Block(
            statements,
            Span::new(start_span, end_span, self.source_id),
        ))
    }

    fn statement(&mut self) -> Failible<Stmt> {
        let next_token = self.next_token()?;
        match next_token.token_kind {
            TokenKind::Put => self.append(next_token.span),
            TokenKind::Shuv => self.assign_array(next_token.span),
            TokenKind::Im => self.loop_statement(next_token.span),
            TokenKind::How => self.function(next_token.span),
            TokenKind::Found => self.return_statement(next_token.span),
            TokenKind::Wtf => self.case(next_token.span),
            TokenKind::O => self.conditional(next_token.span),
            TokenKind::Gtfo => Ok(Stmt {
                span: next_token.span,
                ty: StmtTy::Break,
            }),
            TokenKind::Can => self.import(next_token.span),
            TokenKind::I => {
                if self.peek_token()?.token_kind.eq(&TokenKind::Iz) {
                    self.assignment_or_expr(next_token)
                } else {
                    self.declaration_assignment(next_token.span)
                }
            }
            TokenKind::Visible => self.print(next_token.span),
            TokenKind::Gimmeh => self.input_statement(next_token.span),
            _ => self.assignment_or_expr(next_token),
        }
    }

    fn ident(&mut self) -> Failible<Ident> {
        let id = self.expect(TokenKind::Ident(StrId::default()))?;
        match id.token_kind {
            TokenKind::Ident(s) => Ok(Ident(s, id.span)),
            _ => unreachable!(),
        }
    }

    fn append(&mut self, prev_span: Span) -> Failible<Stmt> {
        let item = self.expr()?;
        self.expect(TokenKind::Into)?;
        let source = self.expr()?;
        Ok(Stmt {
            ty: StmtTy::Append(source, item),
            span: prev_span.combine(&self.current_span),
        })
    }

    fn assign_array(&mut self, prev_span: Span) -> Failible<Stmt> {
        let item = self.expr()?;
        self.expect(TokenKind::Into)?;
        let index = if self.check(&TokenKind::Bak)? {
            Err(false)
        } else if self.check(&TokenKind::Frunt)? {
            Err(true)
        } else {
            Ok(self.expr()?)
        };
        self.expect(TokenKind::Of)?;
        let source = self.expr()?;
        Ok(Stmt {
            ty: StmtTy::SetItem(source, item, index),
            span: prev_span.combine(&self.current_span),
        })
    }

    fn loop_statement(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::In)?;
        self.expect(TokenKind::Yr)?;
        let block_name = self.ident()?;
        let mut fn_id = None;
        if TokenKind::Ident(StrId::default()).eq(&self.peek_token()?.token_kind) {
            let func = self.ident()?;
            self.expect(TokenKind::Yr)?;
            let index = self.ident()?;
            let pred = match self.peek_token()?.token_kind {
                TokenKind::Till => {
                    self.next_token()?;
                    LoopCond::Till(self.expr()?)
                }
                TokenKind::Wile => {
                    self.next_token()?;
                    LoopCond::While(self.expr()?)
                }
                _ => LoopCond::Forever,
            };
            fn_id = Some((func, index, pred));
        }
        self.expect_lines()?;
        let block = self.block(Some(&[TokenKind::Im]))?;
        self.expect(TokenKind::Im)?;
        self.expect(TokenKind::Outta)?;
        self.expect(TokenKind::Yr)?;
        let block_name2 = self.ident()?;

        if block_name2.0 != block_name.0 {
            return Err(
                Diagnostic::build(DiagnosticType::UnmatchedBlockName, block_name2.1)
                    .annotation(
                        Cow::Owned(format!(
                            "the block is called `{}` here",
                            self.lexer.interner.lookup(block_name.0)
                        )),
                        block_name.1,
                    )
                    .annotation(
                        Cow::Owned(format!(
                            "but the block is closed with `{}` here",
                            self.lexer.interner.lookup(block_name2.0)
                        )),
                        block_name2.1,
                    )
                    .into(),
            );
        }

        Ok(Stmt {
            ty: StmtTy::Loop {
                block_name,
                fn_id,
                block,
            },
            span: Span::new(span.s, self.current_span.e, self.source_id),
        })
    }

    fn return_statement(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::Yr)?;
        let expr = self.expr()?;
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::Return(expr),
        })
    }

    fn input_statement(&mut self, span: Span) -> Failible<Stmt> {
        let id = self.ident()?;
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::Input(id),
        })
    }

    fn function(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::Iz)?;
        self.expect(TokenKind::I)?;
        let fn_name = self.ident()?;
        let mut args = Vec::new();
        if self.check(&TokenKind::Yr)? {
            args.push(self.ident()?);
            loop {
                if self.check(&TokenKind::An)? {
                    self.expect(TokenKind::Yr)?;
                    args.push(self.ident()?);
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::Break)?;
        let block = self.block(Some(&[TokenKind::If]))?;
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::U)?;
        self.expect(TokenKind::Say)?;
        self.expect(TokenKind::So)?;
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::FunctionDef(fn_name, args, block),
        })
    }

    fn case_block(&mut self, omg_eaten: bool) -> Failible<(Expr, Block)> {
        if !omg_eaten {
            self.expect(TokenKind::Omg)?;
        }
        let expr = self.expr()?;
        self.expect_lines()?;
        let block = self.block(Some(&[TokenKind::Omgwtf, TokenKind::Omg, TokenKind::Oic]))?;
        Ok((expr, block))
    }

    fn case(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::Question)?;
        self.expect_lines()?;
        if self.check(&TokenKind::Oic)? {
            Ok(Stmt {
                span: Span::new(span.s, self.current_span.e, self.source_id),
                ty: StmtTy::Case(Vec::new(), None),
            })
        } else {
            let mut cases = Vec::new();
            if !TokenKind::Omgwtf.eq(&self.peek_token()?.token_kind) {
                cases.push(self.case_block(false)?);
                loop {
                    let is_omg = self.check(&TokenKind::Omg)?;
                    if is_omg {
                        cases.push(self.case_block(is_omg)?);
                    } else {
                        break;
                    }
                }
            }

            let block = if self.check(&TokenKind::Omgwtf)? {
                self.expect_lines()?;
                let block = self.block(Some(&[TokenKind::Oic]))?;
                Some(block)
            } else {
                None
            };

            self.expect(TokenKind::Oic)?;

            Ok(Stmt {
                span: Span::new(span.s, self.current_span.e, self.source_id),
                ty: StmtTy::Case(cases, block),
            })
        }
    }

    fn conditional(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::Rly)?;
        self.expect(TokenKind::Question)?;
        self.expect(TokenKind::Break)?;

        let ya_rly = if self.check(&TokenKind::Ya)? {
            self.expect(TokenKind::Rly)?;
            self.expect(TokenKind::Break)?;
            let block = self.block(Some(&[TokenKind::Oic, TokenKind::No, TokenKind::Mebbe]))?;
            Some(block)
        } else {
            None
        };

        let mut mebee = Vec::new();

        loop {
            if self.check(&TokenKind::Mebbe)? {
                let expr = self.expr()?;
                self.expect(TokenKind::Break)?;
                let block = self.block(Some(&[TokenKind::Mebbe, TokenKind::No, TokenKind::Oic]))?;
                mebee.push((expr, block));
            } else {
                break;
            }
        }

        let no_wai = if self.check(&TokenKind::No)? {
            self.expect(TokenKind::Wai)?;
            self.expect(TokenKind::Break)?;
            let block = self.block(Some(&[TokenKind::Oic]))?;
            Some(block)
        } else {
            None
        };

        self.expect(TokenKind::Oic)?;

        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::If(ya_rly, mebee, no_wai),
        })
    }

    fn import(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::I)?;
        self.expect(TokenKind::Has)?;
        let ident = self.ident()?;
        self.expect(TokenKind::Question)?;
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::Import(ident),
        })
    }

    fn declaration_assignment(&mut self, span: Span) -> Failible<Stmt> {
        self.expect(TokenKind::Has)?;
        self.expect(TokenKind::A)?;
        let ident = self.ident()?;
        let expr = if self.check(&TokenKind::Itz)? {
            if self.check(&TokenKind::A)? {
                Some(Err(self.ty()?))
            } else {
                Some(Ok(self.expr()?))
            }
        } else {
            None
        };
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::DecAssign(ident, expr),
        })
    }

    fn print(&mut self, span: Span) -> Failible<Stmt> {
        let mut args = vec![self.expr()?];
        while !(self.peek_eq(&TokenKind::Break)? || self.peek_eq(&TokenKind::Bang)?) {
            args.push(self.expr()?);
        }
        let no_newline = self.check(&TokenKind::Bang)?;
        Ok(Stmt {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            ty: StmtTy::Print(args, no_newline),
        })
    }

    fn assignment_or_expr(&mut self, prev: Token) -> Failible<Stmt> {
        let statement_kind = match self.peek_token()?.token_kind {
            TokenKind::Is => {
                self.next_token()?;
                self.expect(TokenKind::Now)?;
                self.expect(TokenKind::A)?;
                StmtTy::MutCast(
                    Ident(
                        match prev.token_kind {
                            TokenKind::Ident(s) => s,
                            _ => unreachable!(),
                        },
                        prev.span,
                    ),
                    self.ty()?,
                )
            }
            TokenKind::R => {
                self.next_token()?;
                let expr = self.expr()?;
                StmtTy::Assignment(
                    Ident(
                        match prev.token_kind {
                            TokenKind::Ident(s) => s,
                            _ => unreachable!(),
                        },
                        prev.span,
                    ),
                    expr,
                )
            }
            _ => {
                let expr = self.expr_inner(Some(prev))?;
                return Ok(Stmt {
                    span: expr.span,
                    ty: StmtTy::Expr(expr),
                });
            }
        };
        Ok(Stmt {
            ty: statement_kind,
            span: Span::new(prev.span.s, self.current_span.e, self.source_id),
        })
    }

    fn ty(&mut self) -> Failible<LolTy> {
        if self.check(&TokenKind::Noob)? {
            return Ok(LolTy::Noob);
        }
        let id = self.ident()?;
        Ok(match self.lexer.interner.lookup(id.0) {
            "TROOF" => LolTy::Troof,
            "YARN" => LolTy::Yarn,
            "NUMBR" => LolTy::Numbr,
            "NUMBAR" => LolTy::Numbar,
            "LIZT" => LolTy::Lizt,
            s => {
                return Err(Diagnostic::build(DiagnosticType::UnknownSymbol, id.1)
                    .annotation(Cow::Owned(format!("`{}` is not a TYPE", s)), id.1)
                    .into())
            }
        })
    }

    fn check(&mut self, token: &TokenKind) -> Failible<bool> {
        if self.peek_token()?.token_kind.eq(token) {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn peek_eq(&mut self, token: &TokenKind) -> Failible<bool> {
        if self.peek_token()?.token_kind.eq(token) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expr(&mut self) -> Failible<Expr> {
        self.expr_inner(None)
    }

    fn expr_binop_of(&mut self, op_ty: OpTy, an_optional: bool) -> Failible<ExprTy> {
        self.expect(TokenKind::Of)?;
        self.expr_binop(op_ty, an_optional)
    }

    fn expr_binop(&mut self, op_ty: OpTy, an_optional: bool) -> Failible<ExprTy> {
        let left = self.expr()?;
        if an_optional {
            self.check(&TokenKind::An)?;
        } else {
            self.expect(TokenKind::An)?;
        }
        let right = self.expr()?;
        Ok(ExprTy::Operator(op_ty, Box::new(left), Box::new(right)))
    }

    fn repeated(&mut self) -> Failible<Vec<Expr>> {
        let mut args = vec![self.expr()?];
        while !self.peek_token()?.token_kind.eq(&TokenKind::Mkay) {
            self.check(&TokenKind::An)?;
            args.push(self.expr()?);
        }
        self.expect(TokenKind::Mkay)?;
        Ok(args)
    }

    fn cmp_ops(&mut self) -> Failible<OpTy> {
        if self.check(&TokenKind::Greter)? {
            let op = if self.check(&TokenKind::Eq)? {
                Ok(OpTy::GTE)
            } else {
                Ok(OpTy::GT)
            };
            self.expect(TokenKind::Then)?;
            return op;
        }

        if self.check(&TokenKind::Les)? {
            let op = if self.check(&TokenKind::Eq)? {
                Ok(OpTy::LTE)
            } else {
                Ok(OpTy::LT)
            };
            self.expect(TokenKind::Then)?;
            return op;
        }

        let next = self.next_token()?;

        Err(Diagnostic::build(DiagnosticType::Syntax, next.span)
            .annotation(
                Cow::Owned(format!(
                    "expected a comparison expression, found {}",
                    next.token_kind
                )),
                next.span,
            )
            .into())
    }

    fn expr_inner(&mut self, prev: Option<Token>) -> Failible<Expr> {
        let to_match = match prev {
            Some(val) => val,
            None => self.next_token()?,
        };

        let kind = match to_match.token_kind {
            TokenKind::Iz => {
                // Comparison
                let left = self.expr()?;
                let op = self.cmp_ops()?;
                let right = self.expr()?;
                ExprTy::Operator(op, Box::new(left), Box::new(right))
            }

            TokenKind::Maek => {
                let expr = self.expr()?;
                self.expect(TokenKind::A)?;
                let ty = self.ty()?;
                ExprTy::Cast(Box::new(expr), ty)
            }
            TokenKind::Ident(id) => ExprTy::Variable(Ident(id, to_match.span)),
            TokenKind::String(s) => ExprTy::String(s),
            TokenKind::InterpStr(s, interps) => ExprTy::InterpStr(s, interps),
            TokenKind::Win => ExprTy::Bool(true),
            TokenKind::Fail => ExprTy::Bool(false),
            TokenKind::Grab => {
                let index = if self.check(&TokenKind::Bak)? {
                    Err(false)
                } else if self.check(&TokenKind::Frunt)? {
                    Err(true)
                } else {
                    Ok(Box::new(self.expr()?))
                };
                self.expect(TokenKind::Frum)?;
                let source = self.expr()?;
                ExprTy::GetItem(Box::new(source), index)
            }

            TokenKind::Noob => ExprTy::Null,

            TokenKind::Sum => self.expr_binop_of(OpTy::Add, false)?,
            TokenKind::Diff => self.expr_binop_of(OpTy::Sub, false)?,
            TokenKind::Quoshunt => self.expr_binop_of(OpTy::Div, false)?,
            TokenKind::Produkt => self.expr_binop_of(OpTy::Mul, false)?,
            TokenKind::Mod => self.expr_binop_of(OpTy::Mod, false)?,

            TokenKind::Biggr => self.expr_binop_of(OpTy::Max, false)?,
            TokenKind::Smallr => self.expr_binop_of(OpTy::Min, false)?,

            TokenKind::Both => {
                if self.check(&TokenKind::Saem)? {
                    self.expr_binop(OpTy::Equal, true)?
                } else {
                    self.expr_binop_of(OpTy::And, true)?
                }
            }
            TokenKind::Either => self.expr_binop_of(OpTy::Or, true)?,
            TokenKind::Won => self.expr_binop_of(OpTy::Xor, true)?,

            TokenKind::Diffrint => self.expr_binop(OpTy::NotEq, true)?,

            TokenKind::Not => ExprTy::UnaryOp(UnOpTy::Not, Box::new(self.expr()?)),
            TokenKind::Langth => {
                self.expect(TokenKind::Of)?;
                ExprTy::UnaryOp(UnOpTy::Length, Box::new(self.expr()?))
            }

            TokenKind::Smoosh => {
                let mut args = vec![self.expr()?];
                while !(self.check(&TokenKind::Mkay)? || self.check(&TokenKind::Break)?) {
                    self.check(&TokenKind::An)?;
                    args.push(self.expr()?);
                }
                self.check(&TokenKind::Mkay)?;
                ExprTy::Concat(args)
            }

            TokenKind::I => {
                self.expect(TokenKind::Iz)?;
                let name = self.ident()?;
                let mut args = Vec::new();
                if self.check(&TokenKind::Yr)? {
                    args.push(self.expr()?);
                    loop {
                        if self.check(&TokenKind::An)? {
                            self.expect(TokenKind::Yr)?;
                            args.push(self.expr()?);
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::Mkay)?;
                ExprTy::FunctionCall(name, args)
            }

            TokenKind::All => {
                self.expect(TokenKind::Of)?;
                let args = self.repeated()?;
                ExprTy::All(args)
            }

            TokenKind::Chain => {
                self.expect(TokenKind::Of)?;
                let args = self.repeated()?;
                ExprTy::List(args)
            }

            TokenKind::Any => {
                self.expect(TokenKind::Of)?;
                let args = self.repeated()?;
                ExprTy::Any(args)
            }

            TokenKind::Number(n1) => {
                let peek = self.peek_token()?;
                let peek_span = peek.span;
                match peek.token_kind {
                    TokenKind::Dot => {
                        self.next_token()?;
                        match self.next_token()?.token_kind {
                            TokenKind::Number(n2) => ExprTy::Float(
                                format!("{}.{}", n1, n2)
                                    .parse::<f64>()
                                    .expect("Invalid floating point"),
                            ),
                            _ => {
                                return Err(Diagnostic::build(
                                    DiagnosticType::Syntax,
                                    to_match.span,
                                )
                                .annotation(
                                    Cow::Borrowed("expected a number after the `.` token"),
                                    peek_span,
                                )
                                .into());
                            }
                        }
                    }
                    _ => ExprTy::Int(n1.parse::<i64>().expect("Invalid integer")),
                }
            }
            _ => {
                return Err(Diagnostic::build(DiagnosticType::Syntax, to_match.span)
                    .annotation(
                        Cow::Owned(format!(
                            "expected an expression, found {}",
                            to_match.token_kind
                        )),
                        to_match.span,
                    )
                    .into())
            }
        };

        Ok(Expr {
            ty: kind,
            span: Span::new(to_match.span.s, self.current_span.e, self.source_id),
        })
    }
}

#[cfg(test)]
mod parse_test {
    use super::*;

    macro_rules! assert_err {
        ($stream: expr, $err_ty: expr, $no_of_annotations: expr, $name: ident) => {
            #[test]
            fn $name() {
                let mut interner = Interner::default();
                let lexer = Lexer::new($stream.chars(), 0, &mut interner);
                let mut parser = Parser::new(lexer);
                let ast = parser.parse();
                match ast {
                    Ok(val) => panic!("Expected Err value, found {:?}", val),
                    Err(e) => {
                        assert_eq!(
                            e.clone().into_inner()[0].annotations.len(),
                            $no_of_annotations
                        );
                        assert_eq!(e.into_inner()[0].ty, $err_ty);
                    }
                }
            }
        };
    }

    macro_rules! assert_ast {
        ($stream: expr, $name: ident, [$($pat: pat,)*]) => {
            #[test]
            #[allow(unused_mut, unused_variables)]
            fn $name() {
                let mut interner = Interner::default();
                let lexer = Lexer::new($stream.chars(), 0, &mut interner);
                let mut parser = Parser::new(lexer);
                let ast = parser.parse();
                let mut ast_iter = ast.expect("Failed to parse").0.into_iter();

                $(match ast_iter.next().unwrap().ty {
                    $pat => {}
                    val => panic!("Unexpected statement: {:?}", val)
                })*
            }
        };
    }

    assert_ast!("HAI 1.4\nKTHXBYE", basic_empty, []);

    assert_ast!(
        "HAI 1.4, ident IS NOW A TROOF, KTHXBYE",
        ident_cast_mut,
        [StmtTy::MutCast(..),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\n\nI HAS A ident ITZ A LIZT\nKTHXBYE",
        lizt_dec,
        [StmtTy::DecAssign(_, Some(Err(LolTy::Lizt))),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nPUT x INTO ARRAY\nKTHXBYE",
        lizt_append,
        [StmtTy::Append(..),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nGRAB FRUNT FRUM ARRAY\nKTHXBYE",
        lizt_grab_frunt,
        [StmtTy::Expr(Expr {ty: ExprTy::GetItem(_, Err(true)), ..}),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nGRAB BAK FRUM ARRAY\nKTHXBYE",
        lizt_grab_bak,
        [StmtTy::Expr(Expr {ty: ExprTy::GetItem(_, Err(false)), ..}),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nGRAB expr FRUM ARRAY\nKTHXBYE",
        lizt_grab_idx,
        [StmtTy::Expr(Expr {ty: ExprTy::GetItem(_, Ok(_)), ..}),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nSHUV X INTO expr OF ARRAY\nKTHXBYE",
        lizt_shuv_idx,
        [StmtTy::SetItem(_, _, Ok(_)),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nSHUV X INTO FRUNT OF ARRAY\nKTHXBYE",
        lizt_shuv_frunt,
        [StmtTy::SetItem(_, _, Err(true)),]
    );

    #[rustfmt::skip]
    assert_ast!(
        "HAI 1.4\nSHUV X INTO BAK OF ARRAY\nKTHXBYE",
        lizt_shuv_bak,
        [StmtTy::SetItem(_, _, Err(false)),]
    );

    assert_ast!(
        "HAI 1.4\n\nI HAS A ident\nKTHXBYE",
        var_dec,
        [StmtTy::DecAssign(..),]
    );

    assert_ast!(
        "HAI 1.4, I HAS A ident ITZ 10, KTHXBYE",
        assign_dec_num,
        [StmtTy::DecAssign(..),]
    );

    assert_ast!(
        "HAI 1.4, I HAS A ident ITZ \"hello\", KTHXBYE",
        assign_dec_string,
        [StmtTy::DecAssign(..),]
    );

    assert_ast!(
        "HAI 1.4, VISIBLE \"hello, world\", KTHXBYE",
        print_string,
        [StmtTy::Print(..),]
    );

    assert_ast!(
        "HAI 1.4, test_id R 10, KTHXBYE",
        assignment_value_integer,
        [StmtTy::Assignment(..),]
    );

    assert_ast!(
        "HAI 1.4, test_id R \"hi\", KTHXBYE",
        assignment_value_string,
        [StmtTy::Assignment(..),]
    );

    assert_ast!(
        r#"HAI 1.4
I IZ UPPIN YR 10 AN YR 10 AN YR 123 MKAY
KTHXBYE"#,
        function_call_3_args,
        [StmtTy::Expr(Expr {
            ty: ExprTy::FunctionCall(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4
I IZ UPPIN YR 10 AN YR 10 MKAY
KTHXBYE"#,
        function_call_2_args,
        [StmtTy::Expr(Expr {
            ty: ExprTy::FunctionCall(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4
I IZ UPPIN MKAY
KTHXBYE"#,
        function_call_no_args,
        [StmtTy::Expr(Expr {
            ty: ExprTy::FunctionCall(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4
I IZ UPPIN YR 10 MKAY
KTHXBYE"#,
        function_call_1_arg,
        [StmtTy::Expr(Expr {
            ty: ExprTy::FunctionCall(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, \"hello\", KTHXBYE",
        expr_string,
        [StmtTy::Expr(Expr {
            ty: ExprTy::String(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4, SMOOSH "hi" " world" MKAY, KTHXBYE"#,
        concat_string_no_an_mkay,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Concat(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4, SMOOSH "hi" AN " world" MKAY, KTHXBYE"#,
        concat_string_mkay,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Concat(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4, SMOOSH "hi" " world", KTHXBYE"#,
        concat_string_no_an,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Concat(..),
            ..
        }),]
    );

    assert_ast!(
        r#"HAI 1.4, SMOOSH "hi" AN " world", KTHXBYE"#,
        concat_string,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Concat(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, 123, KTHXBYE",
        expr_int,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Int(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, CHAIN OF 123 AN 123 AN WIN MKAY, KTHXBYE",
        expr_chain3,
        [StmtTy::Expr(Expr {
            ty: ExprTy::List(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, CHAIN OF 123 AN 123 MKAY, KTHXBYE",
        expr_chain_2,
        [StmtTy::Expr(Expr {
            ty: ExprTy::List(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, CHAIN OF 123 MKAY, KTHXBYE",
        expr_chain1,
        [StmtTy::Expr(Expr {
            ty: ExprTy::List(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ALL OF 123 AN 123 AN WIN MKAY, KTHXBYE",
        expr_all3,
        [StmtTy::Expr(Expr {
            ty: ExprTy::All(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ALL OF 123 AN 123 MKAY, KTHXBYE",
        expr_all2,
        [StmtTy::Expr(Expr {
            ty: ExprTy::All(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ALL OF 123 MKAY, KTHXBYE",
        expr_all1,
        [StmtTy::Expr(Expr {
            ty: ExprTy::All(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ANY OF 123 AN 123 AN WIN MKAY, KTHXBYE",
        expr_any3,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Any(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ANY OF 123 AN 123 MKAY, KTHXBYE",
        expr_any2,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Any(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, ANY OF 123 MKAY, KTHXBYE",
        expr_any1,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Any(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, SUM OF 1 AN 2, KTHXBYE",
        expr_add,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Add, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, DIFF OF 1 AN 2, KTHXBYE",
        expr_sub,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Sub, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, PRODUKT OF 1 AN 2, KTHXBYE",
        expr_mul,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Mul, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, QUOSHUNT OF 1 AN 2, KTHXBYE",
        expr_div,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Div, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, MOD OF 1 AN 2, KTHXBYE",
        expr_mod,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Mod, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, BIGGR OF 1 AN 2, KTHXBYE",
        expr_max,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Max, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, SMALLR OF 1 AN 2, KTHXBYE",
        expr_min,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Min, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, IZ 1 LES EQ THEN 2, KTHXBYE",
        lte_expr,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::LTE, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, IZ 1 GRETER EQ THEN 2, KTHXBYE",
        gte_expr,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::GTE, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, IZ 1 LES THEN 2, KTHXBYE",
        lt_expr,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::LT, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, IZ 1 GRETER THEN 2, KTHXBYE",
        gt_expr,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::GT, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, BOTH SAEM 1 2, KTHXBYE",
        expr_same_no_an,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Equal, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, DIFFRINT 1 2, KTHXBYE",
        expr_diff_no_an,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::NotEq, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, BOTH SAEM 1 AN 2, KTHXBYE",
        expr_same,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::Equal, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, DIFFRINT 1 AN 2, KTHXBYE",
        expr_diff,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Operator(OpTy::NotEq, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, LANGTH OF \"hai\", KTHXBYE",
        expr_len,
        [StmtTy::Expr(Expr {
            ty: ExprTy::UnaryOp(UnOpTy::Length, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, NOT WIN, KTHXBYE",
        expr_not,
        [StmtTy::Expr(Expr {
            ty: ExprTy::UnaryOp(UnOpTy::Not, ..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, 123.123, KTHXBYE",
        expr_float,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Float(..),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, MAEK 10 A NUMBAR, KTHXBYE",
        float_cast,
        [StmtTy::Expr(Expr {
            ty: ExprTy::Cast(_, LolTy::Numbar),
            ..
        }),]
    );

    assert_ast!(
        "HAI 1.4, FOUND YR 10, KTHXBYE",
        return_stmt,
        [StmtTy::Return(..),]
    );

    assert_ast!(
        r#"HAI 1.4
HOW IZ I MULTIPLY YR FIRSTOPERANT AN YR SECONDOPERANT
  FOUND YR FIRSTOPERANT
IF U SAY SO
KTHXBYE"#,
        function_with_two_args,
        [StmtTy::FunctionDef(..),]
    );

    assert_ast!(
        r#"HAI 1.4
HOW IZ I MULTIPLY YR FIRSTOPERANT
  FOUND YR FIRSTOPERANT
IF U SAY SO
KTHXBYE"#,
        function_with_one_arg,
        [StmtTy::FunctionDef(..),]
    );

    assert_ast!(
        r#"HAI 1.4
HOW IZ I MULTIPLY
  FOUND YR 10
IF U SAY SO
KTHXBYE"#,
        function_with_no_args,
        [StmtTy::FunctionDef(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMGWTF
        10
OIC
KTHXBYE"#,
        case_with_omgwtf_block,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
        10
    OMG 12
        20
    OMGWTF
        30
OIC
KTHXBYE"#,
        case_two_with_omgwtf_block,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
        30
    OMG 12
        10
OIC
KTHXBYE"#,
        case_two_block,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
        10
OIC
KTHXBYE"#,
        case_one_block,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMGWTF
OIC
KTHXBYE"#,
        case_with_omgwtf,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
    OMG 12
    OMGWTF
OIC
KTHXBYE"#,
        case_two_with_omgwtf,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
    OMG 12
OIC
KTHXBYE"#,
        case_two,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
    OMG 10
OIC
KTHXBYE"#,
        case_one,
        [StmtTy::Case(..),]
    );

    assert_ast!(
        r#"HAI 1.4
GIMMEH hello
KTHXBYE"#,
        input,
        [StmtTy::Input(..),]
    );

    assert_ast!(
        r#"HAI 1.4
WTF ?
OIC
KTHXBYE"#,
        case_none,
        [StmtTy::Case(..),]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i WILE WIN
IM OUTTA YR block
KTHXBYE"#,
        loop_simple_wile,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i WILE WIN
    GTFO
IM OUTTA YR block
KTHXBYE"#,
        loop_simple_wile_break,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i
IM OUTTA YR block
KTHXBYE"#,
        loop_simple,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i
    GTFO
IM OUTTA YR block
KTHXBYE"#,
        loop_simple_break,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i TIL WIN
IM OUTTA YR block
KTHXBYE"#,
        loop_simple_till,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block UPPIN YR i TIL WIN
    GTFO
IM OUTTA YR block
KTHXBYE"#,
        loop_simple_till_break,
        [StmtTy::Loop { .. },]
    );

    #[rustfmt::skip]
    assert_ast!(
        r#"HAI 1.4
IM IN YR block
IM OUTTA YR block
KTHXBYE"#,
        loop_forever_till_break,
        [StmtTy::Loop { .. },]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    NO WAI
        VISIBLE 20
OIC
KTHXBYE"#,
        if_no_wai,
        [StmtTy::If(..),]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    YA RLY
        VISIBLE 10
    MEBBE WIN
        VISIBLE 123
    MEBBE WIN
        VISIBLE 123
    NO WAI
        VISIBLE 1023
OIC
KTHXBYE"#,
        if_full,
        [StmtTy::If(..),]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    YA RLY
        VISIBLE 10
    MEBBE WIN
        VISIBLE 123
OIC
KTHXBYE"#,
        if_ya_rly_mebbe_single,
        [StmtTy::If(..),]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    YA RLY
        VISIBLE 10
    MEBBE WIN
        VISIBLE 123
    MEBBE WIN
        VISIBLE 123
OIC
KTHXBYE"#,
        if_ya_rly_mebbe_many,
        [StmtTy::If(..),]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    YA RLY
        VISIBLE 10
        VISIBLE 10
    NO WAI
        VISIBLE 20
OIC
KTHXBYE"#,
        if_ya_rly_no_wai,
        [StmtTy::If(..),]
    );

    assert_ast!(
        r#"HAI 1.4
O RLY?
    YA RLY
        VISIBLE 10
        VISIBLE 10
OIC
KTHXBYE"#,
        if_ya_rly,
        [StmtTy::If(..),]
    );

    assert_err!(
        r#"HAI 1.4
IM IN YR bLock UPPIN YR i TIL WIN
IM OUTTA YR block
KTHXBYE"#,
        DiagnosticType::UnmatchedBlockName,
        2,
        mismatched_blocks_loop
    );

    assert_err!(
        "HAI 1.4, ANY OF MKAY, KTHXBYE",
        DiagnosticType::Syntax,
        1,
        any_of_mkay
    );

    assert_err!(
        "HAI 1.4, 123., KTHXBYE",
        DiagnosticType::Syntax,
        1,
        float_missing_num_after_dot
    );

    assert_err!(
        "HAI 1.4, I HAS A test ITZ\nKTHXBYE",
        DiagnosticType::Syntax,
        1,
        expected_expr_found_break
    );
}
