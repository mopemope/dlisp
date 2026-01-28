use crate::ast::Value;
use crate::interpreter::Environment;

mod add;
mod print;
mod sleep;
mod sub;

pub fn install(env: &mut Environment) {
    env.set("+".to_string(), Value::NativeFunc(add::add));
    env.set("-".to_string(), Value::NativeFunc(sub::sub));
    env.set("print".to_string(), Value::NativeFunc(print::print));
    env.set("sleep".to_string(), Value::NativeFunc(sleep::sleep_fn));
}
