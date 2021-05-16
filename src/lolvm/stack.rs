use std::{
    iter::IntoIterator,
    ops::{Index, IndexMut},
};

use crate::lolbc::Value;

pub struct Stack(Vec<Value>);

impl Index<usize> for Stack {
    type Output = Value;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Stack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Stack {
    pub fn new() -> Self {
        Self(vec![Value::Null])
    }
    pub fn push(&mut self, value: Value) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Value {
        self.0.pop().unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn popn(&mut self, n: usize) {
        self.0.truncate(self.0.len() - n);
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
