pub mod prelude {
    pub use super::{Annotation, Diagnostic, DiagnosticType, Diagnostics, Failible, Span};
    pub use std::borrow::Cow;
}

use codespan_reporting::diagnostic;
use std::fmt::{self, Display};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct Span {
    pub s: usize,
    pub e: usize,
    pub file: usize,
}

impl Span {
    pub fn new(s: usize, e: usize, file: usize) -> Span {
        Span { s, e, file }
    }

    pub fn combine(&self, other: &Self) -> Self {
        Span {
            file: self.file,
            s: self.s,
            e: other.e,
        }
    }

    pub fn from_arr(arr: [usize; 3]) -> Self {
        Self {
            s: arr[0],
            e: arr[1],
            file: arr[2],
        }
    }
}

use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;

pub type Failible<T> = Result<T, Diagnostics>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Diagnostics(SmallVec<[Diagnostic; 1]>);

impl Diagnostics {
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
    UnknownSymbol = 5,
    Runtime = 6,
    Scope = 7,
    FunctionArgumentMany = 8,
    Type = 9,
}

impl DiagnosticType {
    pub fn description(&self) -> &'static str {
        match self {
            DiagnosticType::UnexpectedCharacter => "unexpected character while lexing",
            DiagnosticType::InvalidEscapeSequence => "invalid escaped character",
            DiagnosticType::Syntax => "syntax error",
            DiagnosticType::UnmatchedBlockName => "block names specified do not match",
            DiagnosticType::UnknownSymbol => "unknown symbol",
            DiagnosticType::Runtime => "runtime error",
            DiagnosticType::FunctionArgumentMany => "too many funkshion arguments",
            DiagnosticType::Scope => "scope error",
            DiagnosticType::Type => "mismatched types",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiagnosticType::UnexpectedCharacter => "unexpected_character",
            DiagnosticType::InvalidEscapeSequence => "invalid_escape",
            DiagnosticType::Syntax => "syntax",
            DiagnosticType::UnmatchedBlockName => "unmatched_block_name",
            DiagnosticType::UnknownSymbol => "unknown_symbol",
            DiagnosticType::Runtime => "runtime",
            DiagnosticType::FunctionArgumentMany => "funk_arg_many",
            DiagnosticType::Scope => "scope",
            DiagnosticType::Type => "type_error",
        }
    }
}

impl Display for DiagnosticType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "E{:0>3}: {}", *self as usize, self.name())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Annotation {
    pub message: Cow<'static, str>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// A generic error
pub struct Diagnostic {
    pub span: Span,
    pub ty: DiagnosticType,
    pub note: Option<Cow<'static, str>>,
    pub annotations: SmallVec<[Annotation; 1]>,
}

impl Annotation {
    fn into_codespan(self) -> diagnostic::Label<usize> {
        diagnostic::Label::primary(self.span.file, self.span.s..self.span.e)
            .with_message(self.message)
    }
    pub fn new(message: Cow<'static, str>, span: Span) -> Self {
        Self { message, span }
    }
}

impl Diagnostic {
    pub fn build(ty: DiagnosticType, span: Span) -> Self {
        Self {
            annotations: SmallVec::new(),
            note: None,
            ty,
            span,
        }
    }

    pub fn annotation(mut self, message: Cow<'static, str>, span: Span) -> Self {
        self.annotations.push(Annotation::new(message, span));
        self
    }

    pub fn into_codespan(self) -> diagnostic::Diagnostic<usize> {
        let mut initial = diagnostic::Diagnostic::error();
        initial = initial
            .with_message(self.ty.description())
            .with_code(self.ty.to_string())
            .with_labels(
                self.annotations
                    .into_iter()
                    .map(|a| a.into_codespan())
                    .collect(),
            );
        if let Some(note) = self.note {
            initial = initial.with_notes(vec![note.to_string()]);
        }
        initial
    }

    pub fn note(mut self, note: Cow<'static, str>) -> Self {
        self.note = Some(note);
        self
    }
}
