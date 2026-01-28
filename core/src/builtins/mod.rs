use crate::ast::Value;
use crate::interpreter::Environment;

mod add;
mod print;
mod sub;

pub fn install(env: &mut Environment) {
    env.set("+".to_string(), Value::NativeFunc(add::add));
    env.set("-".to_string(), Value::NativeFunc(sub::sub));
    env.set("print".to_string(), Value::NativeFunc(print::print));
}
