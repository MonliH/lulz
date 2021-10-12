#[macro_use]
extern crate derivative;
mod backend;
mod diagnostics;
mod err;
mod frontend;
mod middle;
mod opts;

use clap::Clap;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use std::fs::{read_to_string, write};
use std::io::{self, Read};
use std::{borrow::Cow, mem, process::exit};

use crate::diagnostics::Failible;
use frontend::*;

fn main() {
    let mut opts = opts::Opts::parse();
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

fn pipeline(
    sources: &SimpleFiles<String, String>,
    id: usize,
    mut opts: opts::Opts,
) -> Failible<()> {
    let lexer = lex::Lexer::new(sources.get(id).unwrap().source().chars(), id);
    let mut parser = parse::Parser::new(lexer);
    let mut ast = parser.parse()?;
    ast.opt();
    let mut compiler = middle::LowerCompiler::new(opts.debug_c_gen);
    compiler.compile_start(ast)?;
    let c_code = compiler.get_str();
    if opts.dump_c {
        println!("{}", c_code);
    }
    if let Some(path) = opts.write_c {
        err::report(
            write(path, &c_code),
            Cow::Borrowed("Failed to write c file"),
        );
    }
    let compiler = backend::Compile::new(mem::take(&mut opts.backend));
    compiler.compile(c_code, opts.output, opts.opt, opts.backend_args);
    Ok(())
}
