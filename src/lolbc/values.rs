use std::fmt::{self, Display, Formatter};

pub enum Value {
    Bool(bool),
    Null,
    Float(f64),
    Int(i64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(true) => write!(f, "WIN"),
            Self::Bool(false) => write!(f, "FAIL"),

            Self::Null => write!(f, "NOOB"),

            Self::Float(fl) => write!(f, "NUMBAR `{}`", fl),
            Self::Int(i) => write!(f, "NUMBR `{}`", i),
        }
    }
}

pub struct ValueArray(Vec<Value>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn load(&self, key: usize) -> &Value {
        &self.0[key]
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.0.push(value);
        self.0.len() - 1
    }
}
