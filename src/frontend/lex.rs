use smol_str::SmolStr;
use unicode_names2::character;

use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

use crate::diagnostics::prelude::*;

use super::ast::InterpEntry;

#[derive(Eq, Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Oh my goodness there are so many tokens
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
    Mebbe,
    Wai,
    Gtfo,
    Can,
    I,
    Has,
    Now,
    Itz,
    Is,
    Win,
    Fail,
    Iz,
    An,
    Visible,
    How,
    If,
    U,
    Say,
    Noob,
    So,
    Found,
    Ya,
    No,
    A,
    R,
    Gimmeh,
    Mkay,
    Smoosh,
    Maek,

    Sum,
    Diff,
    Produkt,
    Quoshunt,
    Mod,
    Biggr,
    Smallr,
    Of,

    Both,
    Either,
    Won,
    Not,
    All,
    Any,

    Saem,
    Diffrint,

    Dot,
    Break,
    Question,
    Bang,

    Number(SmolStr),
    String(String),
    /// An interpolated string
    InterpStr(String, Vec<InterpEntry>),
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
                TokenKind::Now => "token `NOW`",
                TokenKind::Omgwtf => "token `OMGWTF`",
                TokenKind::Rly => "token `RLY`",
                TokenKind::O => "token `O`",
                TokenKind::Mebbe => "token `MEBBE`",
                TokenKind::Wai => "token `WAI`",
                TokenKind::Gtfo => "token `GTFO`",
                TokenKind::Can => "token `CAN`",
                TokenKind::I => "token `I`",
                TokenKind::Has => "token `HAS`",
                TokenKind::Itz => "token `ITZ`",
                TokenKind::Win => "token `WIN`",
                TokenKind::Noob => "token `NOOB`",
                TokenKind::Fail => "token `FAIL`",
                TokenKind::Iz => "token `IZ`",
                TokenKind::Is => "token `IS`",
                TokenKind::An => "token `AN`",
                TokenKind::Visible => "token `VISIBLE`",
                TokenKind::How => "token `HOW`",
                TokenKind::If => "token `IF`",
                TokenKind::U => "token `U`",
                TokenKind::Say => "token `SAY`",
                TokenKind::So => "token `SO`",
                TokenKind::Found => "token `FOUND`",
                TokenKind::Ya => "token `YA`",
                TokenKind::No => "token `NO`",
                TokenKind::A => "token `A`",
                TokenKind::R => "token `R`",
                TokenKind::Gimmeh => "token `GIMMEH`",
                TokenKind::Mkay => "token `MKAY`",
                TokenKind::Smoosh => "token `SMOOSH`",
                TokenKind::Maek => "token `MAEK`",

                TokenKind::Sum => "token `SUM`",
                TokenKind::Diff => "token `DIFF`",
                TokenKind::Produkt => "token `PRODUKT`",
                TokenKind::Quoshunt => "token `QUOSHUNT`",
                TokenKind::Mod => "token `MOD`",
                TokenKind::Biggr => "token `BIGGR`",
                TokenKind::Smallr => "token `SMALLR`",
                TokenKind::Of => "token `OF`",

                TokenKind::Both => "token `BOTH`",
                TokenKind::Either => "token `EITHER`",
                TokenKind::Won => "token `WON`",
                TokenKind::Not => "token `NOT`",
                TokenKind::All => "token `ALL`",
                TokenKind::Any => "token `ANY`",

                TokenKind::Saem => "token `SAEM`",
                TokenKind::Diffrint => "token `DIFFRINT`",

                TokenKind::Dot => "token `.`",
                TokenKind::Bang => "token `!`",
                TokenKind::Question => "token `?`",
                TokenKind::Break => "statement separator",

                TokenKind::Number(..) => "number",
                TokenKind::String(..) => "string",
                TokenKind::InterpStr(..) => "interpolated string",
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
            c if Self::is_id_start(c) => return self.ident(c),
            '.' => TokenKind::Dot,
            '?' => TokenKind::Question,
            '!' => TokenKind::Bang,
            '"' => self.eat_string()?,
            ',' => TokenKind::Break,
            '\n' => {
                while self.peek() == '\n' {
                    self.eat();
                }
                TokenKind::Break
            }
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
            .annotation(Cow::Owned(format!("unexpected character `{}`", c)), span)
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
        while predicate(self.peek()) && !self.is_eof() {
            acc.push(self.eat());
        }
        acc
    }

    fn consume_until(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut acc = String::with_capacity(1);
        while predicate(self.peek()) && !self.is_eof() {
            acc.push(self.eat());
        }
        acc
    }

    fn get_str_esc(&mut self, ch: char) -> Failible<(String, Span)> {
        self.eat();
        let esc = self.consume_until(|c| c != ch && c != '"');
        if self.peek() == '"' {
            let span = Span::new(self.position, self.position + 1, self.source_id);
            return Err(Diagnostic::build(
                Level::Error,
                DiagnosticType::InvalidEscapeSequence,
                span,
            )
            .annotation(Cow::Borrowed("unclosed escape"), span)
            .into());
        } else {
            self.eat();
        }
        let span = Span::new(
            self.position - esc.len() - 1,
            self.position - 1,
            self.source_id,
        );

        Ok((esc, span))
    }

    fn validate_id(s: &str) -> bool {
        if s.len() < 1 {
            return false;
        }
        let mut chars = s.chars();
        Self::is_id_start(chars.next().unwrap()) && chars.all(|c| Self::is_id_continue(c))
    }

    fn eat_string(&mut self) -> Failible<TokenKind> {
        let mut acc = String::new();
        let mut interps = Vec::new();
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

                if peeked == '<' {
                    let (esc, span) = self.get_str_esc('>')?;
                    let val = character(&esc).ok_or_else(|| {
                        Diagnostic::build(
                            Level::Error,
                            DiagnosticType::InvalidEscapeSequence,
                            span,
                        )
                        .annotation(
                            Cow::Owned(format!("invalid unicode normative name `{}`", esc)),
                            span,
                        )
                    })?;
                    acc.push(val);
                    continue;
                }

                if peeked == '{' {
                    let (esc, span) = self.get_str_esc('}')?;
                    if Self::validate_id(&esc) {
                        interps.push(InterpEntry(acc.len(), esc, span));
                        continue;
                    } else {
                        return Err(Diagnostic::build(
                            Level::Error,
                            DiagnosticType::InvalidEscapeSequence,
                            span,
                        )
                        .annotation(
                            Cow::Owned(format!("`{}` is not a valid identifier", esc)),
                            span,
                        )
                        .into());
                    }
                }

                if peeked == '(' {
                    let (esc, span) = self.get_str_esc(')')?;
                    let esc_num: u32 = u32::from_str_radix(&esc, 16).map_err(|_| {
                        Diagnostic::build(
                            Level::Error,
                            DiagnosticType::InvalidEscapeSequence,
                            span,
                        )
                        .annotation(Cow::Owned(format!("invalid hex string `{}`", esc)), span)
                    })?;
                    let val = std::char::from_u32(esc_num).ok_or_else(|| {
                        Diagnostic::build(
                            Level::Error,
                            DiagnosticType::InvalidEscapeSequence,
                            span,
                        )
                        .annotation(Cow::Owned(format!("invalid escape hex `{}`", esc)), span)
                    })?;
                    acc.push(val);
                    continue;
                }

                let span = Span::new(self.position, self.position + 1, self.source_id);
                return Err(Diagnostic::build(
                    Level::Error,
                    DiagnosticType::InvalidEscapeSequence,
                    span,
                )
                .annotation(Cow::Owned(format!("invalid escape `:{}`", peeked)), span)
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
                .annotation(Cow::Borrowed("expected `\"`, found end of file"), span)
                .into());
            } else {
                acc.push(next);
            }
        }
        if interps.is_empty() {
            Ok(TokenKind::String(acc))
        } else {
            Ok(TokenKind::InterpStr(acc, interps))
        }
    }

    fn is_not_newline(c: char) -> bool {
        c != '\n'
    }

    fn is_eof(&mut self) -> bool {
        self.peek() == EOF
    }

    fn consume_multiline(&mut self) -> Failible<()> {
        while self.peek() != 'T' && !self.is_eof() {
            self.eat();
        }

        self.eat();
        if self.peek() == 'L' {
            self.eat();
            if self.peek() == 'D' {
                self.eat();
                if self.peek() == 'R' {
                    self.eat();
                    return Ok(());
                }
            }
        }

        self.consume_multiline()
    }

    fn ident(&mut self, first: char) -> Failible<Token> {
        let id = self.consume_while(first, Self::is_id_continue);
        Ok(Token {
            span: Span::new(self.position - id.len(), self.position, self.source_id),
            token_kind: match &id[..] {
                "BTW" => {
                    self.consume_while(first, Self::is_not_newline);
                    return self.next_token();
                }
                "OBTW" => {
                    self.consume_multiline()?;
                    return self.next_token();
                }
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
                "MAEK" => TokenKind::Maek,
                "NOOB" => TokenKind::Noob,
                "O" => TokenKind::O,
                "MEBBE" => TokenKind::Mebbe,
                "WAI" => TokenKind::Wai,
                "IS" => TokenKind::Is,
                "GTFO" => TokenKind::Gtfo,
                "CAN" => TokenKind::Can,
                "I" => TokenKind::I,
                "HAS" => TokenKind::Has,
                "ITZ" => TokenKind::Itz,
                "WIN" => TokenKind::Win,
                "FAIL" => TokenKind::Fail,
                "NOW" => TokenKind::Now,
                "IZ" => TokenKind::Iz,
                "AN" => TokenKind::An,
                "VISIBLE" => TokenKind::Visible,
                "HOW" => TokenKind::How,
                "IF" => TokenKind::If,
                "U" => TokenKind::U,
                "SAY" => TokenKind::Say,
                "SO" => TokenKind::So,
                "FOUND" => TokenKind::Found,
                "YA" => TokenKind::Ya,
                "NO" => TokenKind::No,
                "A" => TokenKind::A,
                "R" => TokenKind::R,
                "GIMMEH" => TokenKind::Gimmeh,
                "MKAY" => TokenKind::Mkay,
                "SMOOSH" => TokenKind::Smoosh,

                "SUM" => TokenKind::Sum,
                "DIFF" => TokenKind::Diff,
                "PRODUKT" => TokenKind::Produkt,
                "QUOSHUNT" => TokenKind::Quoshunt,
                "MOD" => TokenKind::Mod,
                "BIGGR" => TokenKind::Biggr,
                "SMALLR" => TokenKind::Smallr,
                "OF" => TokenKind::Of,

                "BOTH" => TokenKind::Both,
                "EITHER" => TokenKind::Either,
                "WON" => TokenKind::Won,
                "NOT" => TokenKind::Not,
                "ALL" => TokenKind::All,
                "ANY" => TokenKind::Any,

                "SAEM" => TokenKind::Saem,
                "DIFFRINT" => TokenKind::Diffrint,

                _ => TokenKind::Ident(SmolStr::new(id)),
            },
        })
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
    fn comments() {
        assert_map(&[
            ("BTW blah blah blah\n", TokenKind::Break),
            ("BTW aksjdlakwjdlakwjd", TokenKind::Eof),
            (
                "OBTW aksjdlakwjdlakwjd\na  a asd\nasd\nTLD\n TLDAR\n TLDR\n",
                TokenKind::Break,
            ),
            (
                "OBTW aksjdlakwjdlakwjd\na  a asd\nasd\nTLD\n TLDAR\n TLDR",
                TokenKind::Eof,
            ),
        ]);
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
            ("MEBBE", TokenKind::Mebbe),
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
            ("U", TokenKind::U),
            ("SAY", TokenKind::Say),
            ("SO", TokenKind::So),
            ("FOUND", TokenKind::Found),
            ("YA", TokenKind::Ya),
            ("NO", TokenKind::No),
            ("A", TokenKind::A),
            ("R", TokenKind::R),
            ("MAEK", TokenKind::Maek),
            ("GIMMEH", TokenKind::Gimmeh),
            ("MKAY", TokenKind::Mkay),
            ("SMOOSH", TokenKind::Smoosh),
            ("SUM", TokenKind::Sum),
            ("IS", TokenKind::Is),
            ("DIFF", TokenKind::Diff),
            ("PRODUKT", TokenKind::Produkt),
            ("QUOSHUNT", TokenKind::Quoshunt),
            ("MOD", TokenKind::Mod),
            ("BIGGR", TokenKind::Biggr),
            ("SMALLR", TokenKind::Smallr),
            ("NOW", TokenKind::Now),
            ("OF", TokenKind::Of),
            ("BOTH", TokenKind::Both),
            ("EITHER", TokenKind::Either),
            ("WON", TokenKind::Won),
            ("NOT", TokenKind::Not),
            ("ALL", TokenKind::All),
            ("ANY", TokenKind::Any),
            ("SAEM", TokenKind::Saem),
            ("NOOB", TokenKind::Noob),
            ("DIFFRINT", TokenKind::Diffrint),
        ]);
    }

    #[test]
    fn symbols() {
        assert_map(&[
            (".", TokenKind::Dot),
            ("!", TokenKind::Bang),
            ("?", TokenKind::Question),
        ]);
    }

    #[test]
    fn breaks() {
        assert_map(&[
            ("\n", TokenKind::Break),
            (",", TokenKind::Break),
            ("\n\n\n", TokenKind::Break),
        ]);
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
                TokenKind::String("this is a fun string".to_string()),
            ),
            (
                "\":) :: :\" :> :o\"",
                TokenKind::String("\n : \" \t \u{7}".to_string()),
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

    #[test]
    fn token_eq() {
        assert!(TokenKind::Break.eq(&TokenKind::Break));
        assert!(!TokenKind::Break.eq(&TokenKind::Itz));
    }
}
