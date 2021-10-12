use std::mem;

use crate::{diagnostics::Span, frontend::ast::LolTy, middle::StrId};

#[derive(Debug)]
pub struct CBuilder {
    pub fns: Vec<String>,
    pub fn_id: usize,
    pub debug: bool,
    pub main_fn: String,
}

impl CBuilder {
    pub fn new(debug: bool) -> Self {
        Self {
            fns: vec![String::new(), String::new()],
            fn_id: 0,
            debug,
            main_fn: String::new(),
        }
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
    pub fn ws(&mut self, s: &str) {
        self.fns[self.fn_id].push_str(s)
    }

    /// Write character
    pub fn wc(&mut self, c: char) {
        self.fns[self.fn_id].push(c)
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
    pub fn string_lit(&mut self, s: &str) {
        self.literal(LolTy::Yarn, &format!("(char*)\"{}\"", s))
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

    pub fn stdlib(&mut self) {
        self.fns[0].push_str(
            r#"#include "src/clib/lol_runtime.h"
#include "src/clib/lol_opts.h""#,
        );
        self.fn_id += 1;
        self.fns
            .push(format!(include_str!("../clib/main.clol"), self.main_fn));
    }

    pub fn output(self) -> String {
        self.fns.join("\n")
    }
}
