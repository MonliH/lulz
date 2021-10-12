use std::fmt::{self, Display};

pub enum Color {
    Blue,
    Red,
    Bold,
    Reset,
}

impl Color {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Red => "\x1b[31m",
            Self::Reset => "\x1b[0m",
            Self::Blue => "\x1b[34m",
            Self::Bold => "\x1b[1m",
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
