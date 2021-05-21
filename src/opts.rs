use clap::Clap;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clap, Debug)]
#[clap(version = VERSION, author = "Jonathan Li")]
pub struct Opts {
    #[clap(
        short = 'D',
        long = "dump-c",
        about = "Prints the generated C code into stdout"
    )]
    pub dump_c: bool,

    #[clap(
        short = 'W',
        long = "write-c",
        about = "Writes the generated C code to a file"
    )]
    pub write_c: Option<String>,

    #[clap(
        short = 'b',
        long = "backend",
        about = "Set the comipler backend to use",
        default_value = "gcc",
        possible_values = &["gcc", "clang", "tcc"]
    )]
    pub backend: String,

    #[clap(
        short = 'O',
        long = "opt",
        about = "Set the optimization level. Doesn't do anything with --backend=tcc",
        default_value = "0",
        possible_values = &["0", "1", "2", "3", "z"]
    )]
    pub opt: String,

    #[clap(
        short = 'd',
        long = "emit-debug",
        about = "Emit some debug info into the generated C code (extra commenents)"
    )]
    pub debug_c_gen: bool,

    #[clap(short = 'o', long, about = "Output file", default_value = "lol.out")]
    pub output: String,

    #[clap(about = "Input file to compile. Use `-` to read from stdin")]
    pub input: String,

    #[clap(
        short = 'A',
        long = "backend-args",
        about = "Foward these arguments to the backend. You should surround in qoutes if there are spaces",
        allow_hyphen_values = true
    )]
    pub backend_args: Option<String>,
}
