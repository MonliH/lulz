use super::{OpCode, Value, ValueArray};
use crate::diagnostics::Span;

pub type ByteC = Vec<u8>;
pub type ByteCRef<'a> = &'a [u8];

pub type LSpan = Span;

pub struct Positions(Vec<LSpan>);
impl Positions {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, span: LSpan) {
        self.0.push(span);
    }

    pub fn get(&self, idx: usize) -> LSpan {
        self.0[idx]
    }
}

pub struct Chunk {
    pub name: String,
    pub bytecode: ByteC,
    pub pos: Positions,
    pub values: ValueArray,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Self {
            bytecode: Vec::new(),
            values: ValueArray::new(),
            pos: Positions::new(),
            name,
        }
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.values.add_const(value)
    }

    pub fn write_instr(&mut self, instr: u8, line: LSpan) {
        self.pos.add(line);
        self.bytecode.push(instr);
    }

    pub fn write_get_const(&mut self, value: Value, line: LSpan) {
        self.pos.add(line);
        let idx = self.add_const(value);
        if idx <= (u8::MAX as usize) {
            // LoadConst
            self.write_instr(OpCode::LoadConst as u8, line);
            self.write_instr(idx as u8, line);
        } else {
            // LoadConstLong
            self.write_instr(OpCode::LoadConstLong as u8, line);
            self.write_instr((idx >> 16) as u8, line);
            self.write_instr((idx >> 8) as u8, line);
            self.write_instr(idx as u8, line);
        }
    }
}
