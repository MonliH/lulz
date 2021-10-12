use colored::*;
use std::{borrow::Cow, fmt::Display};

pub fn report<T>(val: Result<T, impl Display>, msg: Cow<'static, str>) -> T {
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
