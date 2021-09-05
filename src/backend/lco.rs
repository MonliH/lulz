//! Lazy code objects, as described in "Building JIT
//! compilers for dynamic languages with low development effort":
//!
//! Saleil, B., & Feeley, M. (2018, November). Building JIT
//! compilers for dynamic languages with low development effort.
//! In Proceedings of the 10th ACM SIGPLAN International Workshop
//! on Virtual Machines and Intermediate Languages (pp. 36-46).
use std::{fmt::Debug, mem, ops::Deref, rc::Rc};

use bumpalo::Bump;
use dynasmrt::{
    dynasm,
    x64::{Rq, X64Relocation},
    Assembler, DynasmApi,
};
use hashbrown::HashMap;

use crate::frontend::ast::LolTy;

use super::interner::StrId;

pub type Lazy<'a> = dyn Fn(&mut CompilationCtx) + 'a;
#[derive(Clone)]
pub struct LazyCode<'a>(pub Rc<Lazy<'a>>);

impl<'a> Deref for LazyCode<'a> {
    type Target = Lazy<'a>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Debug)]
pub struct Function {
    pub env: HashMap<StrId, usize>,
    pub stack: Vec<ValueInfo>,
    pub asm: Assembler<X64Relocation>,
}

impl Function {
    pub fn new() -> Self {
        Self {
            env: HashMap::default(),
            stack: Vec::new(),
            asm: Assembler::new().expect("Failed to create assembler"),
        }
    }

    pub fn insert_var(&mut self, name: StrId, idx: usize) {
        self.env.insert(name, idx);
    }

    pub fn push(&mut self, value: ValueInfo) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<ValueInfo> {
        self.stack.pop()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CompilationCtx {
    pub functions: HashMap<StrId, Function>,
    pub fid: StrId,
    pub f: Function,
    pub bump: Bump,
}

impl CompilationCtx {
    pub fn new(main_strid: StrId) -> Self {
        Self {
            functions: HashMap::default(),
            fid: main_strid,
            f: Function::new(),
            bump: Bump::new(),
        }
    }

    pub fn define_var(&mut self, var_id: StrId, value: ValueInfo) {
        let position = self.f.stack.len();
        self.f.push(value);
        self.f.insert_var(var_id, position);
    }

    pub fn var_pos(&self, var_id: &StrId) -> usize {
        *self.f.env.get(var_id).unwrap()
    }

    pub fn var_type(&self, var_id: StrId) -> &ValueInfo {
        &self.f.stack[*self.f.env.get(&var_id).unwrap()]
    }

    pub fn push(&mut self, value: ValueInfo) {
        self.f.push(value);
    }

    pub fn pop(&mut self) -> Option<ValueInfo> {
        self.f.pop()
    }

    pub fn commit_current(&mut self, old_id: StrId, old_fn: Function) -> StrId {
        let current_fn = mem::replace(&mut self.f, old_fn);
        let current_id = self.fid;
        self.functions.insert(current_id, current_fn);
        self.fid = old_id;
        current_id
    }

    pub fn pop_commit(&mut self, old_id: StrId) {
        let old_fn = self.functions.remove(&old_id).unwrap();
        self.commit_current(old_id, old_fn);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ValueInfo {
    ConstBool(bool),
    ConstFloat(f64),
    ConstInt(i32),
    ConstNull,

    Type(LolTy),
}
