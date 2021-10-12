use std::mem;

use crate::{diagnostics::Span, frontend::ast::LolTy, middle::StrId};

#[derive(Debug)]
pub struct CBuilder {
    pub fns: Vec<String>,
    pub fn_id: usize,
    pub debug: bool,
    pub should_emit: bool,
}

impl CBuilder {
    pub fn new(debug: bool) -> Self {
        Self {
            fns: vec![String::new(), String::new()],
            fn_id: 0,
            debug,
            should_emit: true,
        }
    }

    pub fn should_emit(&mut self, should_emit: bool) {
        self.should_emit = should_emit;
    }

    pub fn write_dec(&mut self) -> usize {
        let new = mem::replace(&mut self.fn_id, 1);
        new
    }

    pub fn debug_comment(&mut self, s: &str) {
        if self.debug {
            self.ws("// ");
            self.ws(s);
            self.br();
        }
    }

    pub fn debug_symbol(&mut self, msg: &str, ident: &str, span: Span) {
        self.debug_comment(&format!("{} `{}` at {}:{}", msg, ident, span.s, span.e));
    }

    /// Write semi newline
    pub fn semi(&mut self) {
        self.wc(';');
        self.br()
    }

    /// Write newline
    pub fn br(&mut self) {
        self.wc('\n')
    }

    /// Write string
    #[inline(always)]
    pub fn ws(&mut self, s: &str) {
        if self.should_emit {
            self.fns[self.fn_id].push_str(s)
        }
    }

    /// Write character
    #[inline(always)]
    pub fn wc(&mut self, c: char) {
        if self.should_emit {
            self.fns[self.fn_id].push(c)
        }
    }

    /// Write space
    pub fn wspc(&mut self) {
        self.wc(' ');
    }

    pub fn begin_scope(&mut self) {
        self.ws("{\n")
    }
    pub fn end_scope(&mut self) {
        self.ws("}\n")
    }

    pub fn cast(&mut self, ty: LolTy) {
        self.ws("to_lol_");
        self.ws(ty.as_cast());
    }

    fn literal(&mut self, ty: LolTy, val: &str) {
        self.ws(ty.as_macro());
        self.ws("_VALUE");
        self.wc('(');
        self.ws(val);
        self.wc(')');
    }

    pub fn float(&mut self, float: f64) {
        self.literal(LolTy::Numbar, &format!("{:e}", float))
    }
    pub fn int(&mut self, int: i64) {
        self.literal(LolTy::Numbr, &int.to_string())
    }
    pub fn bool(&mut self, b: bool) {
        self.literal(LolTy::Troof, if b { "1" } else { "0" })
    }
    pub fn null(&mut self) {
        self.ws("NULL_VALUE");
    }
    pub fn function_ptr(&mut self, id: StrId) {
        self.literal(LolTy::Func, &format!("(LolFn)(lol_{}_fn_dyn)", id.get_id()))
    }

    pub fn box_name(&mut self, closure_name: StrId, upvalue_idx: usize) {
        self.ws("lol_box_dyn_ptr(AS_CLOSURE(");
        self.name(closure_name);
        self.ws(")->upvalues[");
        self.ws(&upvalue_idx.to_string());
        self.ws("])");
        self.semi();
    }

    pub fn comma(&mut self) {
        self.ws(", ");
    }

    pub fn ret(&mut self) {
        self.ws("return");
        self.wspc();
    }

    pub fn name(&mut self, name: StrId) {
        self.ws("lol_");
        self.ws(&name.get_id().to_string());
    }

    fn lol_value(&mut self) {
        self.ws("LolValue")
    }

    pub fn lol_case_jmp(&mut self, id: usize) {
        self.ws(&format!("lol_case_{}", id));
    }

    pub fn lol_value_ty(&mut self) {
        self.lol_value();
        self.wspc();
    }

    pub fn fn_dec(&mut self, name: StrId, args: &[StrId]) {
        self.lol_value_ty();
        self.name(name);
        self.ws("_fn");
        self.wc('(');
        let mut args = args.iter();
        if let Some(arg) = args.next() {
            self.lol_value_ty();
            self.name(*arg);
        }
        for arg in args {
            self.ws(", ");
            self.lol_value_ty();
            self.name(*arg);
        }
        self.wc(')');
    }

    pub fn fn_dec_dyn(&mut self, name: StrId) {
        self.ws(&format!(
            include_str!("../clib/dec_dyn_function.clol"),
            name.get_id()
        ));
    }

    pub fn it(&mut self) {
        self.ws("lol_it");
    }

    fn span_ty(&mut self) {
        self.ws("LolSpan");
    }

    pub fn upvalue(&mut self, id: usize) {
        self.ws("*(env[");
        self.ws(&id.to_string());
        self.ws("]->ptr)");
    }

    pub fn closure_name(&mut self, fn_names: impl Iterator<Item = StrId>) -> String {
        format!(
            "lol_{}_fn_closure",
            fn_names
                .map(|i| i.get_id().to_string())
                .collect::<Vec<_>>()
                .join("_")
        )
    }

    pub fn dec_closure(&mut self, fn_name: &str) {
        self.ws(&format!(include_str!("../clib/dec_closure.clol"), fn_name));
    }
    pub fn def_closure(&mut self, len: usize, fn_name: &str) {
        self.ws(&format!(include_str!("../clib/closure.clol"), fn_name, len));
    }

    pub fn span(&mut self, sp: Span) {
        self.wc('(');
        self.span_ty();
        self.wc(')');
        self.wc('{');
        self.ws(&sp.s.to_string());
        self.wc(',');
        self.ws(&sp.e.to_string());
        self.wc('}');
    }

    pub fn stdlib(&mut self, main_fn: &str) {
        self.fns[0].push_str("#include <lol_runtime.h>\n#include <lol_ops.h>");
        self.fn_id += 1;
        self.fns
            .push(format!(include_str!("../clib/main.clol"), main_fn));
    }

    pub fn output(self) -> String {
        self.fns.join("\n")
    }
}
