use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use crate::{diagnostics::prelude::*, frontend::ast::Type};
use Value::*;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Null,
    Float(f64),
    Int(i64),
    // Strings are reference counted
    Str(Rc<str>),
    // Vec<(str_idx, stack_idx)>
    IStr(String, Vec<(usize, usize)>),
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
    /// Returns an `None` if the string is not a valid number, however.
    #[inline(always)]
    pub fn cast_try_num(self) -> Option<Self> {
        Some(match self {
            Int(_) => self,
            Float(_) => self,
            Null => Int(0),
            Bool(b) => Int(b as i64),
            Fun(_) => {
                return None;
            }
            Str(s) => {
                if s.contains('.') {
                    // Parse float
                    if let Ok(f) = s.parse::<f64>() {
                        Float(f)
                    } else {
                        return None;
                    }
                } else {
                    // Parse int
                    if let Ok(f) = s.parse::<i64>() {
                        Int(f)
                    } else {
                        return None;
                    }
                }
            }
            IStr(..) => unreachable!(),
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
            IStr(..) => unreachable!(),
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
            IStr(..) => unreachable!(),
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
            IStr(..) => unreachable!(),
        }
    }

    fn ast_ty(&self) -> Option<Type> {
        match self {
            Bool(..) => Some(Type::Bool),
            Null => Some(Type::Null),
            Float(..) => Some(Type::Float),
            Int(..) => Some(Type::Int),
            Str(..) => Some(Type::Str),
            Fun(..) => None,
            IStr(..) => None,
        }
    }

    fn conv_err(&self, span: Span, expected_ty: &'static str) -> Diagnostics {
        Diagnostic::build(Level::Error, DiagnosticType::Runtime, span)
            .annotation(
                Cow::Owned(format!(
                    "could not convert {} into a {}",
                    self.ty(),
                    expected_ty
                )),
                span,
            )
            .into()
    }

    pub fn cast(self, ty: Type, span: Span) -> Failible<Self> {
        if Some(ty) == self.ast_ty() {
            return Ok(self);
        }
        Ok(match ty {
            Type::Bool => Bool(self.to_bool()),
            Type::Null => Null,
            Type::Float => match self.clone().cast_try_num() {
                Some(Float(f)) => Float(f),
                Some(Int(i)) => Float(i as f64),
                None => return Err(self.conv_err(span, "NUMBAR")),
                _ => unreachable!(),
            },
            Type::Int => match self.clone().cast_try_num() {
                Some(Float(f)) => Int(f as i64),
                Some(Int(i)) => Int(i),
                None => return Err(self.conv_err(span, "NUMBR")),
                _ => unreachable!(),
            },
            Type::Str => Str(self.to_str()),
        })
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
            IStr(..) => unreachable!(),
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
            IStr(s, values) => write!(f, "IYARN `{}` ({:?})", s, values),
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
