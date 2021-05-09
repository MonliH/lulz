pub type Value = f64;
pub struct ValueArray(Vec<Value>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn load(&self, key: usize) -> Value {
        self.0[key]
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.0.push(value);
        self.0.len() - 1
    }
}
