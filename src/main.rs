mod color;
mod diagnostics;
mod err;
mod frontend;
mod middle;
mod opts;

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use std::{
    borrow::Cow,
    fs::read_to_string,
    io::{self, Read},
    process::exit,
};

use crate::diagnostics::Failible;
use frontend::*;

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

fn pipeline(
    sources: &SimpleFiles<String, String>,
    id: usize,
    mut opts: opts::Opts,
) -> Failible<()> {
    let lexer = lex::Lexer::new(sources.get(id).unwrap().source().chars(), id);
    let mut parser = parse::Parser::new(lexer);
    let mut ast = parser.parse()?;

    Ok(())
}
