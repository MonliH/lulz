use std::iter::IntoIterator;

use crate::lolbc::Value;

#[derive(Default)]
pub struct Stack(Vec<Value>);

impl Stack {
    pub fn push(&mut self, value: Value) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Value {
        self.0.pop().unwrap()
    }
}

impl IntoIterator for Stack {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Stack {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
