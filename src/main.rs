mod diagnostics;
mod err;
mod frontend;
mod lolbc;
mod lolvm;
mod middle;

use clap::Clap;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};
use std::fs::read_to_string;
use std::io::{self, Read};
use std::{borrow::Cow, process::exit};

use crate::diagnostics::Failible;
use frontend::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clap, Debug)]
#[clap(version = VERSION, author = "Jonathan Li")]
struct Opts {
    #[clap(short = 'd', long, about = "Prints disassembled lolvm bytecode")]
    disasm: bool,

    #[clap(
        long = "debug-vm",
        about = "Steps through the stack and instructions of the lolvm"
    )]
    debug_vm: bool,

    #[clap(about = "Input file to interpret. Use `-` to read from stdin")]
    input: String,
}

fn main() {
    let mut opts: Opts = Opts::parse();
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

fn pipeline(sources: &SimpleFiles<String, String>, id: usize, opts: Opts) -> Failible<()> {
    let lexer = lex::Lexer::new(sources.get(id).unwrap().source().chars(), id);
    let mut parser = parse::Parser::new(lexer);
    let ast = parser.parse()?;
    let mut bytecode_compiler = middle::BytecodeCompiler::new();
    bytecode_compiler.compile_start(ast)?;
    let mut bytecode: lolbc::Chunk = bytecode_compiler.take_chunk();
    bytecode.opt();
    if opts.disasm {
        lolbc::disasm(&bytecode);
    }
    let mut vm = lolvm::LolVm::new();
    vm.run(bytecode, opts.debug_vm)?;
    Ok(())
}
