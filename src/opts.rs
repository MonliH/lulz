use std::{ffi::OsString, fmt::Write};

pub const HELP: &str = "\
lulz 0.1.0
Jonathan Li

USAGE:
    lulz [FLAGS] [OPTIONS] <input>

ARGS:
    <input>    Input file to compile. Use `-` to read from stdin

FLAGS:
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -o, --output <file>                Output file [default: lol.out]
    --dump-lua <file>                  Dump generated lua code a specified file
    -d, --debug                        Turn debug mode on (for development)
";

pub fn parse() -> Result<Opts, pico_args::Error> {
    let mut args: Vec<_> = std::env::args_os().collect();
    args.remove(0);

    let mut pargs = pico_args::Arguments::from_vec(args);

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Opts {
        output: pargs
            .opt_value_from_str(["-o", "--output"])?
            .unwrap_or_else(|| "lol.out".to_string()),
        dump_lua: pargs.opt_value_from_str("--dump-lua").unwrap(),
        debug: pargs.contains(["-d", "--debug"]),
        input: pargs.free_from_str()?,
    };

    let remaining = pargs.finish();

    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

pub struct Opts {
    pub output: String,
    pub input: String,
    pub dump_lua: Option<String>,
    pub debug: bool
}
