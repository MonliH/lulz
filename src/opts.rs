pub const HELP: &str = "\
lulz 0.1.0
Jonathan Li

USAGE:
    lulz [FLAGS] [OPTIONS] <input>

ARGS:
    <input>    Input file to compile. Use `-` to read from stdin

FLAGS:
    -d, --emit-debug    Emit some debug info into the generated C code (extra commenents)
    -D, --dump-c        Prints the generated C code into stdout
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -b, --backend <backend>
            Set the C compiler backend to use. Examples: gcc, clang, or tcc [default: gcc]

    -A, --backend-args <arguments>
            Foward these arguments to the backend

    -O, --opt <level>
            Set the optimization level [default: 0]
            [possible values: 0, 1, 2, 3, z]

    -o, --output <file>                Output file [default: lol.out]
    -W, --write-c <file>               Writes the generated C code to a file
";

pub fn parse() -> Result<Opts, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Opts {
        dump_c: pargs.contains(["-D", "--dump-c"]),
        write_c: pargs.opt_value_from_str(["-W", "--write-c"])?,
        backend: pargs
            .opt_value_from_str(["-b", "--backend"])?
            .unwrap_or_else(|| "gcc".to_string()),
        opt: pargs
            .opt_value_from_str(["-O", "--opt"])?
            .unwrap_or_else(|| "0".to_string()),
        debug_c_gen: pargs.contains(["-d", "--emit-debug"]),
        output: pargs
            .opt_value_from_str(["-o", "--output"])?
            .unwrap_or_else(|| "lol.out".to_string()),
        input: pargs.free_from_str()?,
        backend_args: pargs.opt_value_from_str(["-A", "--backend-args"])?,
    };

    let remaining = pargs.finish();

    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

pub struct Opts {
    pub dump_c: bool,
    pub write_c: Option<String>,
    pub backend: String,
    pub opt: String,
    pub debug_c_gen: bool,
    pub output: String,
    pub input: String,
    pub backend_args: Option<String>,
}
