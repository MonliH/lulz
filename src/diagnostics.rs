pub mod prelude {
    pub use super::{Diagnostic, DiagnosticType, Failible, Level, Span};
    pub use std::borrow::Cow;
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Span {
    pub s: usize,
    pub e: usize,
    pub file: usize,
}

impl Span {
    pub fn new(s: usize, e: usize, file: usize) -> Span {
        Span { s, e, file }
    }
}

use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;

pub type Failible<T> = Result<T, Diagnostics>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Diagnostics(SmallVec<[Diagnostic; 1]>);

impl Diagnostics {
    pub fn extend(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }

    pub fn push(&mut self, other: Diagnostic) {
        self.0.push(other);
    }

    pub fn inner(&self) -> &SmallVec<[Diagnostic; 1]> {
        &self.0
    }

    pub fn into_inner(self) -> SmallVec<[Diagnostic; 1]> {
        self.0
    }
}

impl From<Diagnostic> for Diagnostics {
    fn from(diagnostic: Diagnostic) -> Self {
        Self(smallvec![diagnostic])
    }
}

impl From<SmallVec<[Diagnostic; 1]>> for Diagnostics {
    fn from(diagnostics: SmallVec<[Diagnostic; 1]>) -> Self {
        Self(diagnostics)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An error type
pub enum DiagnosticType {
    UnexpectedCharacter = 1,
    InvalidEscapeSequence = 2,
    Syntax = 3,
    UnmatchedBlockName = 4,
}

impl DiagnosticType {
    pub fn description(&self) -> &'static str {
        match self {
            DiagnosticType::UnexpectedCharacter => "unexpected character while lexing",
            DiagnosticType::InvalidEscapeSequence => "invalid escaped character",
            DiagnosticType::Syntax => "syntax error",
            DiagnosticType::UnmatchedBlockName => "block names specified do not match",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiagnosticType::UnexpectedCharacter => "unexpected_character",
            DiagnosticType::InvalidEscapeSequence => "invalid_escape",
            DiagnosticType::Syntax => "syntax",
            DiagnosticType::UnmatchedBlockName => "unmatched_block_name",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Annotation {
    pub level: Level,
    pub message: Cow<'static, str>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// A generic error
pub struct Diagnostic {
    pub span: Span,
    pub ty: DiagnosticType,
    pub level: Level,
    pub annotations: SmallVec<[Annotation; 1]>,
}

impl Diagnostic {
    pub fn build(level: Level, ty: DiagnosticType, span: Span) -> Self {
        Self {
            annotations: SmallVec::new(),
            ty,
            span,
            level,
        }
    }

    pub fn into_diagnostics(self) -> Diagnostics {
        Diagnostics(smallvec![self])
    }

    pub fn annotation(mut self, level: Level, message: Cow<'static, str>, span: Span) -> Self {
        self.annotations.push(Annotation {
            level,
            message,
            span,
        });
        self
    }
}
