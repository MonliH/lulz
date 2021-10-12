use super::{bits::Bits, OpCode, Value, ValueArray};
use crate::{
    diagnostics::Span,
    interner::{Interner, StrId},
};

pub type ByteC = Vec<u8>;

pub type LSpan = Span;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct Chunk {
    pub bytecode: ByteC,
    pub pos: Positions,
    pub values: ValueArray,
    pub interner: Interner,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            values: ValueArray::new(),
            pos: Positions::new(),
            interner: Interner::with_capacity(2),
        }
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        self.values.add_const(value)
    }

    pub fn write_instr(&mut self, instr: u8, line: LSpan) {
        self.pos.add(line);
        self.bytecode.push(instr);
    }

    pub fn write_get_const(&mut self, value: Value, line: LSpan) -> Bits {
        let idx: Bits = self.add_const(value).into();
        match idx {
            Bits::U8(idx) => {
                // LoadConst
                self.write_instr(OpCode::LoadConst as u8, line);
                self.write_instr(idx as u8, line);
            }
            Bits::U24(hi, mi, lo) => {
                // LoadConstLong
                self.write_instr(OpCode::LoadConstLong as u8, line);
                self.write_instr(hi, line);
                self.write_instr(mi, line);
                self.write_instr(lo, line);
            }
        }
        idx
    }

    pub fn write_interned(&mut self, s: &str) -> StrId {
        self.interner.intern(s)
    }
}
