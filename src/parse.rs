use std::iter::Peekable;

use crate::ast::*;
use crate::diagnostics::prelude::*;
use crate::lex::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }

    fn next_token(&mut self) -> Failible<Token> {
        self.lexer.next().unwrap()
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
        let mut block = self.block()?;
        self.expect(TokenKind::Kthxbye);
        self.expect(TokenKind::Eof);
        Ok(block)
    }

    fn version(&mut self) -> Failible<()> {
        todo!()
    }

    fn block(&mut self) -> Failible<Block> {
        todo!()
    }
}
