mod ast;
mod diagnostics;
mod err;
mod interpret;
mod lex;
mod parse;

use clap::Clap;
use std::borrow::Cow;
use std::fs::read_to_string;
use std::io::{self, Read};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clap, Debug)]
#[clap(version = VERSION, author = "Jonathan Li")]
struct Opts {
    #[clap(about = "Input file to interpret. Use `-` to read from stdin.")]
    input: String,
    #[clap(short = 'V', long)]
    version: bool,
}

fn main() {
    let opts: Opts = Opts::parse();
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
}
