use mlua::Lua;

pub mod io {
    pub const LUA_PRINT: &str = "_lulz_print";
    pub const LUA_PRINTLN: &str = "_lulz_println";
}

macro_rules! include_module {
    ($name: expr) => {
        include_str!(concat!("builtins/", concat!($name, ".lua")))
    }
}

fn run_str(lj: &Lua, s: &str) {
    lj.load(s).exec().unwrap();
}

pub fn register_modules(lj: &Lua) {
    run_str(lj, include_module!("io"))
}
