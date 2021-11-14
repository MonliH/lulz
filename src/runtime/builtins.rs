use mlua::Lua;
use super::errors::register_raise_error;

pub mod io {
    pub const LUA_PRINT: &str = "_lulz_print";
    pub const LUA_PRINTLN: &str = "_lulz_println";
}

pub mod null {
    pub const LUA_NEW_NULL: &str = "_lulz_NewNull";
    pub const LUA_CHECK_VARIABLE: &str = "_lulz_check_variable";
}

pub mod ops {
    pub const LUA_ADD: &str = "_lulz_add";
    pub const LUA_SUB: &str = "_lulz_sub";
    pub const LUA_MUL: &str = "_lulz_mul";
    pub const LUA_DIV: &str = "_lulz_div";
    pub const LUA_MOD: &str = "_lulz_mod";
    pub const LUA_AND: &str = "_lulz_and";
    pub const LUA_OR: &str = "_lulz_or";
    pub const LUA_EQ: &str = "_lulz_eq";
    pub const LUA_NEQ: &str = "_lulz_neq";
    pub const LUA_GT: &str = "_lulz_gt";
    pub const LUA_LT: &str = "_lulz_lt";
    pub const LUA_GTE: &str = "_lulz_gte";
    pub const LUA_LTE: &str = "_lulz_lte";
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
    // Order of loading is important
    register_raise_error(lj);

    // Load null first because it overrides the `type` built-in function (evil, I know)
    run_str(lj, include_module!("null"));

    run_str(lj, include_module!("io"));
    run_str(lj, include_module!("ops"));
}
