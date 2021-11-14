use std::process::exit;

use codespan_reporting::{
    diagnostic::{self, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use mlua::Lua;

use crate::diagnostics::{DiagnosticType, Diagnostics, Span};
use crate::sourcemap::SOURCEMAP;

fn raise_error_at(span: Span, msg: &str) -> ! {
    let codespan_err = diagnostic::Diagnostic::new(diagnostic::Severity::Error)
        .with_message(msg)
        .with_code(DiagnosticType::Runtime.to_string())
        .with_labels(vec![Label::primary(span.file, (span.s)..(span.e))]);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();

    let mut guard = SOURCEMAP.write().unwrap();
    let sourcemap = std::mem::replace(&mut *guard, SimpleFiles::new());
    term::emit(&mut writer.lock(), &config, &sourcemap, &codespan_err)
        .expect("Failed to write error");

    // Failed
    exit(1)
}

pub fn register_raise_error(lj: &Lua) {
    let globals = lj.globals();
    globals.set(
        "_ffi_lulz_error",
        lj.create_function(|lua, (span_arr, msg): ([usize; 3], String)| {
            let span = Span::from_arr(span_arr);
            raise_error_at(span, &msg);
            Ok(())
        }).unwrap(),
    ).unwrap();
}

pub fn raise_errors(es: Diagnostics) -> ! {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();

    let mut guard = SOURCEMAP.write().unwrap();
    let sourcemap = std::mem::replace(&mut *guard, SimpleFiles::new());
    for e in es.into_inner().into_iter() {
        term::emit(
            &mut writer.lock(),
            &config,
            &sourcemap,
            &(e.into_codespan()),
        )
        .expect("Failed to write error");
    }

    // Failed
    exit(1)
}
