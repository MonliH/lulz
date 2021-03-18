use smol_str::SmolStr;
use std::iter::Peekable;

use crate::ast::*;
use crate::diagnostics::prelude::*;
use crate::lex::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    current_span: Span,
    source_id: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            current_span: Span::new(0, 0, lexer.source_id),
            source_id: lexer.source_id,
            lexer: lexer.peekable(),
        }
    }

    fn next_token(&mut self) -> Failible<Token> {
        let tok = self.lexer.next().unwrap()?;
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
        match unsafe { raw_self.as_mut() }.unwrap().lexer.peek().unwrap() {
            Ok(t) => Ok(t),
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
            Err(
                Diagnostic::build(Level::Error, DiagnosticType::Syntax, next.span)
                    .annotation(
                        Level::Error,
                        Cow::Owned(format!("expected {}, but found {}", token, next.token_kind)),
                        next.span,
                    )
                    .into(),
            )
        }
    }

    pub fn parse(&mut self) -> Failible<Block> {
        self.expect(TokenKind::Hai)?;
        self.version()?;
        self.expect(TokenKind::Break)?;
        let block = self.block()?;
        self.expect(TokenKind::Kthxbye)?;
        self.expect(TokenKind::Eof)?;
        Ok(block)
    }

    fn version(&mut self) -> Failible<()> {
        self.expect(TokenKind::Number(SmolStr::default()))?;
        self.expect(TokenKind::Dot)?;
        self.expect(TokenKind::Number(SmolStr::default()))?;
        Ok(())
    }

    fn block(&mut self) -> Failible<Block> {
        let start_span = self.current_span.s;
        let mut statements = vec![self.statement()?];
        loop {
            if TokenKind::Break.eq(&self.peek_token()?.token_kind) {
                statements.push(self.statement()?);
            } else {
                break;
            }
        }
        let end_span = self.current_span.e;

        Ok(Block(
            statements,
            Span::new(start_span, end_span, self.source_id),
        ))
    }

    fn statement(&mut self) -> Failible<Statement> {
        let next_token = self.next_token()?;
        match next_token.token_kind {
            TokenKind::Im => self.loop_statement(next_token.span),
            TokenKind::How => self.function(next_token.span),
            TokenKind::Found => self.return_statement(next_token.span),
            TokenKind::Wtf => self.case(next_token.span),
            TokenKind::O => self.conditional(next_token.span),
            TokenKind::Gtfo => Ok(Statement {
                span: next_token.span,
                statement_kind: StatementKind::Break,
            }),
            TokenKind::Can => self.import(next_token.span),
            TokenKind::I => self.declaration_assignment(next_token.span),
            TokenKind::Visible => self.print(next_token.span),
            _ => self.assignment_or_expr(next_token),
        }
    }

    fn ident(&mut self) -> Failible<Ident> {
        let id = self.expect(TokenKind::Ident(SmolStr::default()))?;
        match id.token_kind {
            TokenKind::Ident(s) => Ok(Ident(s, id.span)),
            _ => unreachable!(),
        }
    }

    fn loop_statement(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::In)?;
        self.expect(TokenKind::Yr)?;
        let block_name = self.ident()?;
        let func = self.ident()?;
        self.expect(TokenKind::Yr)?;
        let index = self.ident()?;
        let pred = match self.peek_token()?.token_kind {
            TokenKind::Till => Some((true, self.expr()?)),
            TokenKind::Wile => Some((false, self.expr()?)),
            _ => None,
        };
        self.expect(TokenKind::Break)?;
        let block = self.block()?;
        self.expect(TokenKind::Im)?;
        self.expect(TokenKind::Outta)?;
        self.expect(TokenKind::Yr)?;
        let block_name2 = self.ident()?;

        if block_name2.0 != block_name.0 {
            return Err(Diagnostic::build(
                Level::Error,
                DiagnosticType::UnmatchedBlockName,
                block_name2.1,
            )
            .annotation(
                Level::Error,
                Cow::Owned(format!("the block is called `{}` here", &block_name.0)),
                block_name.1,
            )
            .annotation(
                Level::Error,
                Cow::Owned(format!(
                    "but the block is closed with `{}` here",
                    &block_name2.0
                )),
                block_name2.1,
            )
            .into());
        }

        Ok(Statement {
            statement_kind: StatementKind::Loop {
                block_name,
                func,
                index,
                pred,
                block,
            },
            span: Span::new(span.s, self.current_span.e, self.source_id),
        })
    }

    fn return_statement(&mut self, span: Span) -> Failible<Statement> {
        let expr = self.expr()?;
        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::Return(expr),
        })
    }

    fn function(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::Iz)?;
        self.expect(TokenKind::I)?;
        let fn_name = self.ident()?;
        let mut args = Vec::new();
        if TokenKind::Yr.eq(&self.peek_token()?.token_kind) {
            args.push(self.ident()?);
            loop {
                if TokenKind::An.eq(&self.peek_token()?.token_kind) {
                    self.expect(TokenKind::Yr)?;
                    args.push(self.ident()?);
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::Break)?;
        let block = self.block()?;
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::You)?;
        self.expect(TokenKind::Say)?;
        self.expect(TokenKind::So)?;
        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::FunctionDef(fn_name, args, block),
        })
    }

    fn case(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::Question)?;
        self.expect(TokenKind::Break)?;
        let mut cases = Vec::new();
        loop {
            if TokenKind::Break.eq(&self.peek_token()?.token_kind) {
                self.expect(TokenKind::Omg)?;
                let expr = self.expr()?;
                self.expect(TokenKind::Break)?;
                let block = self.block()?;
                cases.push((expr, block));
            } else {
                break;
            }
        }

        let block = if TokenKind::Omgwtf.eq(&self.peek_token()?.token_kind) {
            self.expect(TokenKind::Break)?;
            let block = self.block()?;
            Some(block)
        } else {
            None
        };

        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::Case(cases, block),
        })
    }

    fn conditional(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::Rly)?;
        self.expect(TokenKind::Question)?;
        self.expect(TokenKind::Break)?;

        let ya_rly = if TokenKind::Ya.eq(&self.peek_token()?.token_kind) {
            self.expect(TokenKind::Rly)?;
            let block = self.block()?;
            Some(block)
        } else {
            None
        };

        let mut mebee = Vec::new();

        loop {
            if TokenKind::Mebee.eq(&self.peek_token()?.token_kind) {
                let expr = self.expr()?;
                self.expect(TokenKind::Break)?;
                let block = self.block()?;
                mebee.push((expr, block));
            } else {
                break;
            }
        }

        let no_wai = if TokenKind::No.eq(&self.peek_token()?.token_kind) {
            self.expect(TokenKind::Wai)?;
            let block = self.block()?;
            Some(block)
        } else {
            None
        };

        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::If(ya_rly, mebee, no_wai),
        })
    }

    fn import(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::I)?;
        self.expect(TokenKind::Has)?;
        let ident = self.ident()?;
        self.expect(TokenKind::Question)?;
        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::Import(ident),
        })
    }

    fn declaration_assignment(&mut self, span: Span) -> Failible<Statement> {
        self.expect(TokenKind::I)?;
        self.expect(TokenKind::Has)?;
        self.expect(TokenKind::A)?;
        let ident = self.ident()?;
        let expr = if TokenKind::Itz.eq(&self.next_token()?.token_kind) {
            Some(self.expr()?)
        } else {
            None
        };
        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::DecAssign(ident, expr),
        })
    }

    fn print(&mut self, span: Span) -> Failible<Statement> {
        let expr = self.expr()?;
        Ok(Statement {
            span: Span::new(span.s, self.current_span.e, self.source_id),
            statement_kind: StatementKind::Print(expr),
        })
    }

    fn assignment_or_expr(&mut self, prev: Token) -> Failible<Statement> {
        match self.peek_token()?.token_kind {
            TokenKind::R => {
                self.next_token()?;
                let expr = self.expr()?;
                Ok(Statement {
                    statement_kind: StatementKind::Assignment(
                        Ident(
                            match prev.token_kind {
                                TokenKind::Ident(s) => s,
                                _ => unreachable!(),
                            },
                            prev.span,
                        ),
                        expr,
                    ),
                    span: Span::new(prev.span.s, self.current_span.e, self.source_id),
                })
            }
            _ => {
                let expr = self.expr_inner(Some(prev))?;
                Ok(Statement {
                    span: expr.span,
                    statement_kind: StatementKind::Expr(expr),
                })
            }
        }
    }

    fn expr(&mut self) -> Failible<Expr> {
        self.expr_inner(None)
    }

    fn expr_inner(&mut self, prev: Option<Token>) -> Failible<Expr> {
        let to_match = match prev {
            Some(val) => val,
            None => self.next_token()?,
        };

        let kind = match to_match.token_kind {
            TokenKind::Ident(id) => ExprKind::Variable(Ident(id, to_match.span)),
            TokenKind::String(s) => ExprKind::String(s),
            TokenKind::Win => ExprKind::Boolean(true),
            TokenKind::Fail => ExprKind::Boolean(false),
            TokenKind::Number(n1) => {
                let peek = self.peek_token()?;
                let peek_span = peek.span;
                match peek.token_kind {
                    TokenKind::Dot => {
                        self.next_token()?;
                        match self.next_token()?.token_kind {
                            TokenKind::Number(n2) => ExprKind::Float(
                                format!("{}.{}", n1, n2)
                                    .parse::<f64>()
                                    .expect("Invalid floating point"),
                            ),
                            _ => {
                                return Err(Diagnostic::build(
                                    Level::Error,
                                    DiagnosticType::Syntax,
                                    to_match.span,
                                )
                                .annotation(
                                    Level::Error,
                                    Cow::Borrowed("expected a number after the `.` token"),
                                    peek_span,
                                )
                                .into());
                            }
                        }
                    }
                    _ => ExprKind::Int(n1.parse::<i64>().expect("Invalid integer")),
                }
            }
            _ => {
                return Err(
                    Diagnostic::build(Level::Error, DiagnosticType::Syntax, to_match.span)
                        .annotation(
                            Level::Error,
                            Cow::Borrowed("expected an expression here"),
                            to_match.span,
                        )
                        .into(),
                )
            }
        };

        Ok(Expr {
            expr_kind: kind,
            span: Span::new(to_match.span.s, self.current_span.e, self.source_id),
        })
    }
}

#[cfg(test)]
mod parse_test {
    use super::*;

    fn assert_err(stream: &'static str, err_ty: DiagnosticType, no_of_annotation: usize) {
        let lexer = Lexer::new(stream.chars(), 0);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse();
        match ast {
            Ok(val) => panic!("Expected Err value, found {:?}", val),
            Err(e) => {
                assert_eq!(e.inner()[0].annotations.len(), no_of_annotation);
                assert_eq!(e.inner()[0].ty, err_ty);
            }
        }
    }

    macro_rules! assert_ast {
        ($stream: expr, $name: ident, [$($pat: pat,)*]) => {
            #[test]
            fn $name() {
                let lexer = Lexer::new($stream.chars(), 0);
                let mut parser = Parser::new(lexer);
                let ast = parser.parse();
                let mut ast_iter = ast.expect("Failed to parse").0.into_iter();

                $(match ast_iter.next().unwrap().statement_kind {
                    $pat => {}
                    val => panic!("Unexpected statement: {:?}", val)
                })*
            }
        };
    }

    assert_ast!("HAI 1.4\nKTHXBYE", basic_empty, []);
    assert_ast!(
        "HAI 1.4\nI HAS A ident\nKTHXBYE",
        var_dec,
        [StatementKind::DecAssign(..),]
    );
    assert_ast!(
        "HAI 1.4, I HAS A ident ITZ 10, KTHXBYE",
        assign_dec,
        [StatementKind::DecAssign(..),]
    );
}
