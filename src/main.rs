mod backend;
mod color;
mod diagnostics;
mod err;
mod frontend;
mod opts;
mod runtime;
mod sourcemap;

use crate::backend::interner::Interner;
use crate::diagnostics::Failible;
use crate::runtime::builtins::register_modules;
use crate::sourcemap::SOURCEMAP;
use backend::translator::Translator;
use frontend::*;
use std::{
    borrow::Cow,
    fs::read_to_string,
    io::{self, Read}
};

use mlua::Lua;

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
    let id = SOURCEMAP.write().unwrap().add(std::mem::take(&mut opts.input), source);
    match pipeline(id, opts) {
        Ok(()) => {}
        Err(es) => {

        }
    };
}


fn pipeline(id: usize, opts: opts::Opts) -> Failible<()> {
    let mut interner = Interner::default();
    let guard = SOURCEMAP.read().unwrap();
    let lexer = lex::Lexer::new(
        guard.get(id).unwrap().source().chars(),
        id,
        &mut interner,
    );
    let mut parser = parse::Parser::new(lexer);
    let ast = parser.parse()?;
    std::mem::drop(guard);

    let mut translator = Translator::new(interner);
    translator.block(ast)?;

    eprintln!("{}", translator.code);
    let lj = Lua::new();
    register_modules(&lj);
    lj.load(&translator.code)
        .exec()
        .expect("Generated lua code should not crash");

    Ok(())
}
