use crate::lolbc::Chunk;
use std::mem;

trait BCOptPass {
    fn apply(&mut self, bytecode: Vec<u8>) -> Vec<u8>;
}

#[inline]
fn apply_passes(passes: &mut [&mut dyn BCOptPass], bytecode: Vec<u8>) -> Vec<u8> {
    let mut current = bytecode;
    for pass in passes.iter_mut() {
        current = (*pass).apply(current);
    }
    current
}

impl Chunk {
    pub fn opt(&mut self) {
        let owned = mem::take(&mut self.bytecode);
        self.bytecode = apply_passes(&mut [&mut DoubleIt], owned);
    }
}

pub struct DoubleIt;
impl BCOptPass for DoubleIt {
    fn apply(&mut self, bytecode: Vec<u8>) -> Vec<u8> {
        bytecode
    }
}

#[allow(dead_code)]
fn replace(mut source: Vec<u8>, from: &[u8], to: &[u8]) -> Vec<u8> {
    let from_len = from.len();
    let to_len = to.len();

    let mut i = 0;
    while i + from_len <= source.len() {
        if source[i..].starts_with(from) {
            source.splice(i..i + from_len, to.iter().cloned());
            i += to_len;
        } else {
            i += 1;
        }
    }

    source
}
