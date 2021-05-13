mod frame;
mod interner;
mod lol_vm;
mod stack;
pub use frame::CallFrame;
pub use interner::{Interner, StrId};
pub use lol_vm::LolVm;
pub use stack::Stack;
