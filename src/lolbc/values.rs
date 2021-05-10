use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use crate::diagnostics::prelude::*;
use Value::*;

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Null,
    Float(f64),
    Int(i64),
    // Strings are reference counted
    Str(Rc<str>),
}

impl Value {
    /// Convert to number if possible, otherwise leaving the value untouched.
    /// Returns an `Err()` if the string is not a valid number, however.
    pub fn cast_try_num(self, span: Span, expr_name: &str) -> Failible<Self> {
        Ok(match self {
            Bool(b) => Int(b as i64),
            Str(s) => {
                if s.contains('.') {
                    // Parse float
                    if let Ok(f) = s.parse::<f64>() {
                        Float(f)
                    } else {
                        return Err(Diagnostic::build(
                            Level::Error,
                            DiagnosticType::Runtime,
                            span,
                        )
                        .annotation(
                            Cow::Owned(format!(
                                "could not convert {} YARN to a NUMBAR",
                                expr_name
                            )),
                            span,
                        )
                        .into());
                    }
                } else {
                    // Parse int
                    if let Ok(f) = s.parse::<i64>() {
                        Int(f)
                    } else {
                        return Err(Diagnostic::build(
                            Level::Error,
                            DiagnosticType::Runtime,
                            span,
                        )
                        .annotation(
                            Cow::Owned(format!("could not convert {} YARN to a NUMBR", expr_name)),
                            span,
                        )
                        .into());
                    }
                }
            }
            _ => self,
        })
    }

    pub fn ty(&self) -> &'static str {
        match self {
            Bool(..) => "TROOF",
            Null => "NOOB",
            Float(..) => "NUMBAR",
            Int(..) => "NUMBR",
            Str(..) => "YARN",
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Bool(b) => *b,
            Null => false,
            Float(f) => f == &0.0,
            Int(i) => i == &0,
            Str(s) => !s.is_empty(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Bool(true) => write!(f, "WIN"),
            Bool(false) => write!(f, "FAIL"),

            Null => write!(f, "NOOB"),

            Float(fl) => write!(f, "NUMBAR `{}`", fl),
            Int(i) => write!(f, "NUMBR `{}`", i),

            Str(s) => write!(f, "YARN `{}`", s),
        }
    }
}

#[derive(Default)]
pub struct ValueArray(Vec<Value>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn load(&self, key: usize) -> Value {
        self.0[key].clone()
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.0.push(value);
        self.0.len() - 1
    }
}
