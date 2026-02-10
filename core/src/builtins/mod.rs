use crate::ast::Value;
use crate::environment::Environment;

pub mod add;
pub mod eq;
pub mod gt;
pub mod io;
pub mod list;
pub mod list_ops;
pub mod lt;
pub mod map;
pub mod mul;
pub mod print;
pub mod sleep;
pub mod sub;
pub mod vector;

pub fn install(env: &mut Environment) {
    env.set("+".to_string(), Value::NativeFunc(add::add));
    env.set("-".to_string(), Value::NativeFunc(sub::sub));
    env.set("*".to_string(), Value::NativeFunc(mul::mul));
    env.set(">".to_string(), Value::NativeFunc(gt::gt));
    env.set("<".to_string(), Value::NativeFunc(lt::lt));
    env.set("=".to_string(), Value::NativeFunc(eq::eq));
    env.set("print".to_string(), Value::NativeFunc(print::print));
    env.set("sleep".to_string(), Value::NativeFunc(sleep::sleep_fn));
    env.set("read-file".to_string(), Value::NativeFunc(io::read_file));
    env.set("list".to_string(), Value::NativeFunc(list::list));
    env.set("vector".to_string(), Value::NativeFunc(vector::vector));
    env.set("nth".to_string(), Value::NativeFunc(vector::nth));
    env.set("count".to_string(), Value::NativeFunc(vector::count));
    env.set("conj".to_string(), Value::NativeFunc(vector::conj));

    env.set("hash-map".to_string(), Value::NativeFunc(map::hash_map));
    env.set("get".to_string(), Value::NativeFunc(map::get));
    env.set("assoc".to_string(), Value::NativeFunc(map::assoc));
    env.set("dissoc".to_string(), Value::NativeFunc(map::dissoc));
    env.set("keys".to_string(), Value::NativeFunc(map::keys));
    env.set("vals".to_string(), Value::NativeFunc(map::vals));
    env.set("contains?".to_string(), Value::NativeFunc(map::contains_q));

    // List operations
    env.set("not".to_string(), Value::NativeFunc(list_ops::not));
    env.set("car".to_string(), Value::NativeFunc(list_ops::car));
    env.set("first".to_string(), Value::NativeFunc(list_ops::car));
    env.set("cdr".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("rest".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("cons".to_string(), Value::NativeFunc(list_ops::cons));
}
