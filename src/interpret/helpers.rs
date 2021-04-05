use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

use smol_str::SmolStr;

use crate::{
    ast::{Block, Ident},
    diagnostics::prelude::*,
};

type INT = i32;
type FLOAT = f32;

#[derive(Debug, Clone)]
pub enum Prim {
    Int(INT),
    Float(FLOAT),
    Bool(bool),
    String(String),
    Null,
    Function(Vec<Ident>, Block),
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Null,
    Bool,
    Int,
    Float,
    Str,
    Function,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Null => "NOOB",
                Self::Bool => "TROOF",
                Self::Int => "NUMBR",
                Self::Float => "NUMBAR",
                Self::Str => "YARN",
                Self::Function => "INSTRUCTUION",
            }
        )
    }
}

impl fmt::Display for Prim {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if let Self::String(s) = self {
                s
            } else {
                unreachable!()
            }
        )
    }
}

impl Prim {
    pub fn ty(&self) -> Type {
        match self {
            Self::Int(..) => Type::Int,
            Self::Bool(..) => Type::Bool,
            Self::Float(..) => Type::Float,
            Self::String(..) => Type::Str,
            Self::Function(..) => Type::Function,
            Self::Null => Type::Null,
        }
    }

    fn cast_err<T>(&self, err: Result<T, impl fmt::Display>, ty: Type, span: Span) -> Failible<T> {
        match err {
            Ok(t) => Ok(t),
            Err(e) => {
                let mut s = e.to_string();
                if !s.is_empty() {
                    s = format!("\n{}", s)
                }
                Err(Diagnostic::build(Level::Error, DiagnosticType::Cast, span)
                    .annotation(
                        Cow::Owned(format!("failed to cast `{}` to `{}`{}", self.ty(), ty, s)),
                        span,
                    )
                    .into())
            }
        }
    }

    pub fn cast(self, implicit: bool, ty: Type, span: Span) -> Failible<Self> {
        if let Self::Function(..) = self {
            return self.cast_err(Err(""), ty, span);
        }

        Ok(match ty {
            Type::Null => Self::Null,
            Type::Bool => {
                let val = match self {
                    Self::String(s) if s.is_empty() => false,
                    Self::Int(0) => false,
                    Self::Float(f) if f == 0.0 => false,
                    Self::Null => false,
                    Self::Bool(b) => b,
                    _ => true,
                };
                Self::Bool(val)
            }
            Type::Int => {
                let val = match self {
                    Self::Bool(b) => b as INT,
                    Self::Int(i) => i,
                    Self::Float(f) => f.trunc() as INT,
                    Self::String(ref s) => self.cast_err(s.parse(), ty, span)?,
                    Self::Null if !implicit => 0,
                    _ => return self.cast_err(Err(""), ty, span),
                };
                Self::Int(val)
            }
            Type::Float => {
                let val = match self {
                    Self::Bool(b) => (b as INT) as FLOAT,
                    Self::Int(i) => i as FLOAT,
                    Self::Float(f) => f,
                    Self::String(ref s) => self.cast_err(s.parse(), ty, span)?,
                    Self::Null if !implicit => 0.0,
                    _ => return self.cast_err(Err(""), ty, span),
                };
                Self::Float(val)
            }
            Type::Str => {
                let val = match self {
                    Self::Bool(b) => if b { "WIN" } else { "FAIL" }.to_string(),
                    Self::Int(i) => i.to_string(),
                    Self::Float(f) => format!("{:.2}", f),
                    Self::String(s) => s,
                    Self::Null if !implicit => "NOOB".to_string(),
                    _ => return self.cast_err(Err(""), ty, span),
                };
                Self::String(val)
            }
            _ => return self.cast_err(Err(""), ty, span),
        })
    }
}

pub struct Ctx {
    pub sym_tab: HashMap<SmolStr, Prim>,
    pub it: Prim,
}

impl Ctx {
    pub fn new() -> Self {
        let sym_tab = HashMap::new();
        Self {
            sym_tab,
            it: Prim::Null,
        }
    }

    pub fn lookup_str(&self, s: &SmolStr) -> Option<&Prim> {
        self.sym_tab.get(s)
    }

    pub fn lookup(&self, id: &Ident) -> Failible<&Prim> {
        match self.lookup_str(&id.0) {
            Some(p) => Ok(p),
            None => Err(
                Diagnostic::build(Level::Error, DiagnosticType::UnknownSymbol, id.1)
                    .annotation(Cow::Owned(format!("undefined symbol `{}`", id.0)), id.1)
                    .into(),
            ),
        }
    }
}
