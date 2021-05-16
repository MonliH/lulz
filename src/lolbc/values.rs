use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use crate::diagnostics::prelude::*;
use Value::*;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Null,
    Float(f64),
    Int(i64),
    // Strings are reference counted
    Str(Rc<str>),
    Fun(usize),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Null, Null) => true,
            (Float(f1), Float(f2)) => f1 == f2,
            (Float(f), Int(i)) | (Int(i), Float(f)) => &(*i as f64) == f,
            (Int(i1), Int(i2)) => i1 == i2,
            (Str(s1), Str(s2)) => s1 == s2,
            (Fun(f1), Fun(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    /// Convert to number if possible, otherwise leaving the value untouched.
    /// Returns an `Err()` if the string is not a valid number, however.
    pub fn cast_try_num(self, span: Span, expr_name: &str) -> Failible<Self> {
        Ok(match self {
            Bool(b) => Int(b as i64),
            Fun(_) => {
                return Err(Diagnostic::build(Level::Error, DiagnosticType::Type, span)
                    .annotation(
                        Cow::Borrowed("this value is a FUNKSHON, not a NUMBAR or NUMBR"),
                        span,
                    )
                    .into())
            }
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
            Fun(..) => "FUNKSHON",
        }
    }

    pub fn to_str(&self) -> Rc<str> {
        match self {
            Bool(b) => if *b { "WIN" } else { "FAIL" }.into(),
            Null => "NOOB".into(),
            Float(f) => f.to_string().into(),
            Int(i) => i.to_string().into(),
            Str(s) => Rc::clone(&s),
            Fun(id) => format!("<FUNKSHON at {:#x}>", id).into(),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Bool(b) => *b,
            Null => false,
            Float(f) => f != &0.0,
            Int(i) => i != &0,
            Str(s) => !s.is_empty(),
            Fun(_) => true,
        }
    }

    pub fn disp(&self) -> Cow<'static, str> {
        match self {
            Bool(true) => Cow::Borrowed("WIN"),
            Bool(false) => Cow::Borrowed("FAIL"),

            Null => Cow::Borrowed("NOOB"),

            Str(s) => Cow::Owned(s.to_string()),
            Int(i) => Cow::Owned(i.to_string()),
            Float(fl) => Cow::Owned(fl.to_string()),
            Fun(id) => Cow::Owned(format!("<FUNKSHON at {:#x}>", id)),
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
            Fun(id) => write!(f, "FUNKSHON at `{:#x}`", id),
        }
    }
}

#[derive(Default, Debug)]
pub struct ValueArray(Vec<Value>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Vec::new())
    }

    pub fn load(&self, key: usize) -> Value {
        self.0[key].clone()
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.0.push(value);
        self.0.len() - 1
    }
}
