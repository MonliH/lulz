use colored::*;
use std::borrow::Cow;
use std::error::Error;

pub fn report<T>(val: Result<T, impl Error>, msg: Cow<'static, str>) -> T {
    match val {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{}: {}: {}",
                "lulz".color("blue").bold(),
                msg.as_ref().color("red").bold(),
                (&e.to_string()).color("red")
            );
            std::process::exit(1);
        }
    }
}
