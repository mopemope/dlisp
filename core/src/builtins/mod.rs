use crate::ast::Value;
use crate::environment::Environment;

pub mod add;
pub mod eq;
pub mod gt;
pub mod lt;
pub mod mul;
pub mod print;
pub mod sleep;
pub mod sub;

pub fn install(env: &mut Environment) {
    env.set("+".to_string(), Value::NativeFunc(add::add));
    env.set("-".to_string(), Value::NativeFunc(sub::sub));
    env.set("*".to_string(), Value::NativeFunc(mul::mul));
    env.set(">".to_string(), Value::NativeFunc(gt::gt));
    env.set("<".to_string(), Value::NativeFunc(lt::lt));
    env.set("=".to_string(), Value::NativeFunc(eq::eq));
    env.set("print".to_string(), Value::NativeFunc(print::print));
    env.set("sleep".to_string(), Value::NativeFunc(sleep::sleep_fn));
}
