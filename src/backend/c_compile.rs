use crate::err;
use std::{
    borrow::Cow,
    io::{self, Write},
    process::{exit, Child, Command, Stdio},
    str::FromStr,
};

pub enum Backend {
    Clang,
    Gcc,
    Tcc,
}

impl Backend {
    fn compiler_cmd(&self) -> &'static str {
        match self {
            Self::Clang => "clang",
            Self::Gcc => "gcc",
            Self::Tcc => "tcc",
        }
    }

    fn command(&self, opt: &str, output: &str, args: Option<&str>) -> io::Result<Child> {
        let mut proc = Command::new(self.compiler_cmd());
        proc.arg(&format!("-O{}", opt)).args(&[
            "-xc",
            "-o",
            output,
            "-",
            "src/clib/lol_runtime.c",
            "src/clib/lol_opts.c",
        ]);
        if let Some(arg) = args {
            proc.arg(arg);
        }
        proc.stdin(Stdio::piped()).stdout(Stdio::inherit()).spawn()
    }
}

impl FromStr for Backend {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "clang" => Ok(Self::Clang),
            "gcc" => Ok(Self::Gcc),
            "tcc" => Ok(Self::Tcc),
            _ => Err("invalid backend"),
        }
    }
}

pub struct Compile(Backend);

impl Compile {
    pub fn new(backend: &str) -> Self {
        Self(Backend::from_str(backend).expect("Invalid backend"))
    }

    pub fn compile(&self, source: String, output: String, opt: String, args: Option<String>) {
        let mut child = err::report(
            self.0.command(&opt, &output, args.as_deref()),
            Cow::Owned(format!(
                "failed to spawn compiler `{}`",
                self.0.compiler_cmd()
            )),
        );
        let child_stdin = err::report(
            child
                .stdin
                .as_mut()
                .ok_or("failed to write to backend stdin"),
            Cow::Borrowed("failed to pipe"),
        );
        err::report(
            child_stdin.write_all(source.as_bytes()),
            Cow::Borrowed("failed to write to stdin"),
        );
        drop(child_stdin);

        let output = err::report(child.wait_with_output(), Cow::Borrowed("compiler failed"));
        if !output.status.success() {
            exit(1);
        }
    }
}
