use crate::err;
use std::{
    borrow::Cow,
    ffi::OsString,
    io::Write,
    process::{exit, Command, Stdio},
};

pub struct Compile(String);

impl Compile {
    pub fn new(backend: String) -> Self {
        Self(backend)
    }

    pub fn compile(&self, source: String, output: String, opt: String, args: Vec<OsString>) {
        let mut proc = Command::new(&self.0);
        proc.arg(&format!("-O{}", opt))
            .args(&["-xc", "-o", &output, "-", "-llulzrt"])
            .args(&args);
        let mut child = err::report(
            proc.stdin(Stdio::piped()).stdout(Stdio::inherit()).spawn(),
            Cow::Owned(format!("failed to spawn compiler `{}`", &self.0)),
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
