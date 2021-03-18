use smol_str::SmolStr;

use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

use crate::diagnostics::prelude::*;

#[derive(Eq, Debug, PartialEq, Clone)]
pub enum TokenKind {
    Hai,
    Kthxbye,
    Im,
    In,
    Yr,
    Till,
    Wile,
    Outta,
    Wtf,
    Oic,
    Omg,
    Omgwtf,
    Rly,
    O,
    Mebee,
    Wai,
    Gtfo,
    Can,
    I,
    Has,
    Itz,
    Win,
    Fail,
    Iz,
    An,
    Visible,
    How,
    If,
    You,
    Say,
    So,
    Found,
    Ya,
    No,
    A,
    R,

    Dot,
    Break,
    Question,

    Number(SmolStr),
    String(SmolStr),
    Ident(SmolStr),

    Eof,
}

impl TokenKind {
    pub fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::Hai => "token `HAI`",
                TokenKind::Kthxbye => "token `KTHXBYE`",
                TokenKind::Im => "token `IM`",
                TokenKind::In => "token `IN`",
                TokenKind::Yr => "token `YR`",
                TokenKind::Till => "token `TILL`",
                TokenKind::Wile => "token `WILE`",
                TokenKind::Outta => "token `OUTTA`",
                TokenKind::Wtf => "token `WTF`",
                TokenKind::Oic => "token `OIC`",
                TokenKind::Omg => "token `OMG`",
                TokenKind::Omgwtf => "token `OMGWTF`",
                TokenKind::Rly => "token `RLY`",
                TokenKind::O => "token `O`",
                TokenKind::Mebee => "token `MEBEE`",
                TokenKind::Wai => "token `WAI`",
                TokenKind::Gtfo => "token `GTFO`",
                TokenKind::Can => "token `CAN`",
                TokenKind::I => "token `I`",
                TokenKind::Has => "token `HAS`",
                TokenKind::Itz => "token `ITZ`",
                TokenKind::Win => "token `WIN`",
                TokenKind::Fail => "token `FAIL`",
                TokenKind::Iz => "token `IZ`",
                TokenKind::An => "token `AN`",
                TokenKind::Visible => "token `VISIBLE`",
                TokenKind::How => "token `HOW`",
                TokenKind::If => "token `IF`",
                TokenKind::You => "token `YOU`",
                TokenKind::Say => "token `SAY`",
                TokenKind::So => "token `SO`",
                TokenKind::Found => "token `FOUND`",
                TokenKind::Ya => "token `YA`",
                TokenKind::No => "token `NO`",
                TokenKind::A => "token `A`",
                TokenKind::R => "token `R`",

                TokenKind::Dot => "token `.`",
                TokenKind::Question => "token `?`",
                TokenKind::Break => "statement separator",

                TokenKind::Number(..) => "number",
                TokenKind::String(..) => "string",
                TokenKind::Ident(..) => "identifier",

                TokenKind::Eof => "end of file",
            }
        )
    }
}

#[derive(Eq, Debug, PartialEq)]
pub struct Token {
    pub token_kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    stream: Peekable<Chars<'a>>,
    position: usize,
    pub source_id: usize,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Failible<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_token())
    }
}

const EOF: char = '\0';
const ESCAPES: &[(char, char)] = &[
    (':', ':'),
    ('"', '"'),
    (')', '\n'),
    ('>', '\t'),
    ('o', '\u{7}'),
];

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>, source_id: usize) -> Self {
        Self {
            stream: chars.peekable(),
            source_id,
            position: 0,
        }
    }

    fn next_token(&mut self) -> Failible<Token> {
        let prev_pos = self.position;
        let kind = match self.eat() {
            c if Self::is_id_start(c) => self.ident(c),
            '.' => TokenKind::Dot,
            '?' => TokenKind::Question,
            '"' => self.eat_string()?,
            '\n' | ',' => TokenKind::Break,
            '\t' | ' ' => return self.next_token(),
            '\0' => TokenKind::Eof,
            '-' => {
                let c = self.peek();
                match self.next_token()?.token_kind {
                    TokenKind::Number(s) => TokenKind::Number(SmolStr::new(format!("-{}", s))),
                    _ => {
                        return Err(Self::lexer_err(
                            c,
                            Span::new(prev_pos, self.position, self.source_id),
                        ))
                    }
                }
            }
            c if c.is_ascii_digit() => {
                TokenKind::Number(SmolStr::new(self.consume_while(c, |c| c.is_ascii_digit())))
            }
            c => {
                let span = Span::new(prev_pos, self.position, self.source_id);
                return Err(Self::lexer_err(c, span));
            }
        };

        Ok(Token {
            token_kind: kind,
            span: Span::new(prev_pos, self.position, self.source_id),
        })
    }

    fn lexer_err(c: char, span: Span) -> Diagnostics {
        Diagnostic::build(Level::Error, DiagnosticType::UnexpectedCharacter, span)
            .annotation(
                Level::Error,
                Cow::Owned(format!("unexpected character `{}`", c)),
                span,
            )
            .into()
    }

    #[inline]
    /// Go forward one character in the character stream
    fn eat(&mut self) -> char {
        self.position += 1;
        self.stream.next().unwrap_or(EOF)
    }

    #[inline]
    /// Go forward one character without peeking
    fn peek(&mut self) -> char {
        *self.stream.peek().unwrap_or(&EOF)
    }

    #[inline]
    fn is_id_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    #[inline]
    fn is_id_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn consume_while(&mut self, first: char, predicate: impl Fn(char) -> bool) -> String {
        let mut acc = String::with_capacity(1);
        acc.push(first);
        while predicate(self.peek()) {
            acc.push(self.eat());
        }
        acc
    }

    fn eat_string(&mut self) -> Failible<TokenKind> {
        let mut acc = String::new();
        'main: loop {
            let next = self.eat();
            if next == ':' {
                let peeked = self.peek();
                for (k, v) in ESCAPES {
                    if peeked == *k {
                        self.eat();
                        acc.push(*v);
                        // Skip over everything else
                        continue 'main;
                    }
                }

                let span = Span::new(self.position, self.position + 1, self.source_id);
                return Err(Diagnostic::build(
                    Level::Error,
                    DiagnosticType::InvalidEscapeSequence,
                    span,
                )
                .annotation(
                    Level::Error,
                    Cow::Owned(format!("invalid escape `\\{}`", peeked)),
                    span,
                )
                .into());
            } else if next == '"' {
                break;
            } else if next == EOF {
                let span = Span::new(self.position, self.position + 1, self.source_id);
                return Err(Diagnostic::build(
                    Level::Error,
                    DiagnosticType::UnexpectedCharacter,
                    span,
                )
                .annotation(
                    Level::Error,
                    Cow::Borrowed("expected `\"`, found end of file"),
                    span,
                )
                .into());
            } else {
                acc.push(next);
            }
        }
        Ok(TokenKind::String(SmolStr::new(acc)))
    }

    fn ident(&mut self, first: char) -> TokenKind {
        let id = self.consume_while(first, Self::is_id_continue);
        match &id[..] {
            "HAI" => TokenKind::Hai,
            "KTHXBYE" => TokenKind::Kthxbye,
            "IM" => TokenKind::Im,
            "IN" => TokenKind::In,
            "YR" => TokenKind::Yr,
            "TILL" => TokenKind::Till,
            "WILE" => TokenKind::Wile,
            "OUTTA" => TokenKind::Outta,
            "WTF" => TokenKind::Wtf,
            "OIC" => TokenKind::Oic,
            "OMG" => TokenKind::Omg,
            "OMGWTF" => TokenKind::Omgwtf,
            "RLY" => TokenKind::Rly,
            "O" => TokenKind::O,
            "MEBEE" => TokenKind::Mebee,
            "WAI" => TokenKind::Wai,
            "GTFO" => TokenKind::Gtfo,
            "CAN" => TokenKind::Can,
            "I" => TokenKind::I,
            "HAS" => TokenKind::Has,
            "ITZ" => TokenKind::Itz,
            "WIN" => TokenKind::Win,
            "FAIL" => TokenKind::Fail,
            "IZ" => TokenKind::Iz,
            "AN" => TokenKind::An,
            "VISIBLE" => TokenKind::Visible,
            "HOW" => TokenKind::How,
            "IF" => TokenKind::If,
            "YOU" => TokenKind::You,
            "SAY" => TokenKind::Say,
            "SO" => TokenKind::So,
            "FOUND" => TokenKind::Found,
            "YA" => TokenKind::Ya,
            "NO" => TokenKind::No,
            "A" => TokenKind::A,
            "R" => TokenKind::R,
            _ => TokenKind::Ident(SmolStr::new(id)),
        }
    }
}

#[cfg(test)]
mod lexer_test {
    use super::*;

    fn assert_err(map: &[(&'static str, DiagnosticType)]) {
        for (source, err) in map.iter() {
            let mut lexer = Lexer::new(source.chars(), 0);
            assert_eq!(
                lexer.next().unwrap().map_err(|e| e.inner()[0].ty),
                Err(err.clone())
            );
        }
    }

    fn assert_map(map: &[(&'static str, TokenKind)]) {
        for (source, token) in map.iter() {
            let mut lexer = Lexer::new(source.chars(), 0);
            assert_eq!(
                lexer.next().unwrap().map(|t| t.token_kind),
                Ok(token.clone())
            );
        }
    }

    #[test]
    fn keywords() {
        assert_map(&[
            ("HAI", TokenKind::Hai),
            ("KTHXBYE", TokenKind::Kthxbye),
            ("IM", TokenKind::Im),
            ("IN", TokenKind::In),
            ("YR", TokenKind::Yr),
            ("TILL", TokenKind::Till),
            ("WILE", TokenKind::Wile),
            ("OUTTA", TokenKind::Outta),
            ("WTF", TokenKind::Wtf),
            ("OIC", TokenKind::Oic),
            ("OMG", TokenKind::Omg),
            ("OMGWTF", TokenKind::Omgwtf),
            ("RLY", TokenKind::Rly),
            ("O", TokenKind::O),
            ("MEBEE", TokenKind::Mebee),
            ("WAI", TokenKind::Wai),
            ("GTFO", TokenKind::Gtfo),
            ("CAN", TokenKind::Can),
            ("I", TokenKind::I),
            ("HAS", TokenKind::Has),
            ("ITZ", TokenKind::Itz),
            ("WIN", TokenKind::Win),
            ("FAIL", TokenKind::Fail),
            ("IZ", TokenKind::Iz),
            ("AN", TokenKind::An),
            ("VISIBLE", TokenKind::Visible),
            ("IF", TokenKind::If),
            ("YOU", TokenKind::You),
            ("SAY", TokenKind::Say),
            ("SO", TokenKind::So),
            ("FOUND", TokenKind::Found),
            ("YA", TokenKind::Ya),
            ("NO", TokenKind::No),
            ("A", TokenKind::A),
            ("R", TokenKind::R),
        ]);
    }

    #[test]
    fn symbols() {
        assert_map(&[(".", TokenKind::Dot), ("?", TokenKind::Question)]);
    }

    #[test]
    fn breaks() {
        assert_map(&[("\n", TokenKind::Break), (",", TokenKind::Break)]);
    }

    #[test]
    fn ignore_whitespace() {
        assert_map(&[("\t VISIBLE", TokenKind::Visible)]);
    }

    #[test]
    fn ident() {
        assert_map(&[(
            "helloWorldIdent",
            TokenKind::Ident(SmolStr::new("helloWorldIdent")),
        )]);
    }

    #[test]
    fn string() {
        assert_map(&[
            (
                "\"this is a fun string\"",
                TokenKind::String(SmolStr::new("this is a fun string")),
            ),
            (
                "\":) :: :\" :> :o\"",
                TokenKind::String(SmolStr::new("\n : \" \t \u{7}")),
            ),
        ]);
    }

    #[test]
    fn numbers() {
        assert_map(&[
            ("1234567809", TokenKind::Number(SmolStr::new("1234567809"))),
            (
                "-1234567809",
                TokenKind::Number(SmolStr::new("-1234567809")),
            ),
        ]);
    }

    #[test]
    fn errs() {
        assert_err(&[
            ("\":k\"", DiagnosticType::InvalidEscapeSequence),
            ("\"", DiagnosticType::UnexpectedCharacter),
            ("}", DiagnosticType::UnexpectedCharacter),
            ("-asf", DiagnosticType::UnexpectedCharacter),
        ]);
    }
}
