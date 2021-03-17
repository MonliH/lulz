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

    fn parse(&mut self) -> Failible<Block> {
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
            TokenKind::How => self.function(),
            TokenKind::Found => self.return_statement(next_token.span),
            TokenKind::Wtf => self.case(),
            TokenKind::O => self.conditional(),
            TokenKind::Gtfo => Ok(Statement {
                span: next_token.span,
                statement_kind: StatementKind::Break,
            }),
            TokenKind::Can => self.import(),
            TokenKind::I => self.declaration_assignment(),
            TokenKind::Visible => self.print(),
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

    fn break_statement(&mut self, span: Span) -> Failible<Statement> {
        Ok(Statement {
            span,
            statement_kind: StatementKind::Break,
        })
    }

    fn function(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn case(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn conditional(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn import(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn declaration_assignment(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn print(&mut self) -> Failible<Statement> {
        todo!()
    }

    fn assignment_or_expr(&mut self, prev: Token) -> Failible<Statement> {
        todo!()
    }

    fn expr(&mut self) -> Failible<Expr> {
        todo!()
    }
}
