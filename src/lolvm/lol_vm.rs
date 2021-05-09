use crate::lolbc::Chunk;

pub struct LolVm {}

impl LolVm {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, chunk: Chunk) {
        eprintln!("Running...");
    }
}
