mod backend;
mod color;
mod diagnostics;
mod err;
mod frontend;
mod opts;

use crate::backend::interner::Interner;
use crate::backend::translate::{dump_hex, translate_ast};
use crate::diagnostics::Failible;
use backend::lco::{CompilationCtx, LazyCode};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use dynasm::dynasm;
use dynasmrt::DynasmApi;
use frontend::*;
use libc::c_void;
use std::rc::Rc;
use std::{
    borrow::Cow,
    fs::read_to_string,
    io::{self, Read},
    mem,
    process::exit,
};

fn main() {
    let mut opts = err::report(
        opts::parse().map_err(|e| {
            eprint!("{}", opts::HELP);
            e
        }),
        Cow::Borrowed("Failed to parse arguments"),
    );
    let source: String = if &opts.input == "-" {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        err::report(
            handle.read_to_string(&mut buffer),
            Cow::Borrowed("Failed to read from stdin"),
        );
        buffer
    } else {
        err::report(
            read_to_string(&opts.input),
            Cow::Borrowed("Failed to read file"),
        )
    };
    let mut sources = SimpleFiles::new();
    let id = sources.add(std::mem::take(&mut opts.input), source);
    match pipeline(&sources, id, opts) {
        Ok(()) => {}
        Err(es) => {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = term::Config::default();

            for e in es.into_inner().into_iter() {
                term::emit(&mut writer.lock(), &config, &sources, &(e.into_codespan()))
                    .expect("Failed to write error");
            }

            // Failed
            exit(1);
        }
    };
}

fn pipeline(sources: &SimpleFiles<String, String>, id: usize, opts: opts::Opts) -> Failible<()> {
    let mut interner = Interner::default();
    let lexer = lex::Lexer::new(sources.get(id).unwrap().source().chars(), id, &mut interner);
    let mut parser = parse::Parser::new(lexer);
    let ast = parser.parse()?;
    let main_strid = interner.intern("");

    let mut ctx = CompilationCtx::new(main_strid);
    let start = ctx.f.asm.offset();

    translate_ast(
        &ast.0,
        LazyCode(Rc::new(|ctx| {
            dynasm!( ctx.f.asm
            // Return 0
            ; xor rax, rax
            ; ret
            );
            ctx.f.asm.commit().unwrap();
            let main_buf = ctx.f.asm.reader();
            let main_fn: extern "sysv64" fn() -> c_void =
                unsafe { mem::transmute(main_buf.lock().ptr(start)) };
            main_fn();
        })),
    )(&mut ctx);

    Ok(())
}
