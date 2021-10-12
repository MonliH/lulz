use crate::color::Color;
use std::{borrow::Cow, fmt::Display};

pub fn report<T>(val: Result<T, impl Display>, msg: Cow<'static, str>) -> T {
    match val {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{bl}{bo}lulz{rst}: {red}{bo}{err_ty}{rst}: {err_msg}",
                bl = Color::Blue,
                bo = Color::Bold,
                red = Color::Red,
                rst = Color::Reset,
                err_msg = e.to_string(),
                err_ty = msg,
            );
            std::process::exit(1);
        }
    }
}
