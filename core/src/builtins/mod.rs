use crate::ast::Value;
use crate::environment::Environment;

pub mod add;
pub mod cmp;
pub mod div;
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
pub mod string_ops;
pub mod sub;
pub mod type_ops;
pub mod vector;

pub fn install(env: &mut Environment) {
    // Arithmetic
    env.set("+".to_string(), Value::NativeFunc(add::add));
    env.set("-".to_string(), Value::NativeFunc(sub::sub));
    env.set("*".to_string(), Value::NativeFunc(mul::mul));
    env.set("/".to_string(), Value::NativeFunc(div::div));
    env.set("%".to_string(), Value::NativeFunc(div::modulo));
    env.set("mod".to_string(), Value::NativeFunc(div::modulo));

    // Comparison
    env.set(">".to_string(), Value::NativeFunc(gt::gt));
    env.set("<".to_string(), Value::NativeFunc(lt::lt));
    env.set("=".to_string(), Value::NativeFunc(eq::eq));
    env.set(">=".to_string(), Value::NativeFunc(cmp::gte));
    env.set("<=".to_string(), Value::NativeFunc(cmp::lte));
    env.set("/=".to_string(), Value::NativeFunc(cmp::neq));

    // I/O
    env.set("print".to_string(), Value::NativeFunc(print::print));
    env.set("sleep".to_string(), Value::NativeFunc(sleep::sleep_fn));
    env.set("read-file".to_string(), Value::NativeFunc(io::read_file));

    // List
    env.set("list".to_string(), Value::NativeFunc(list::list));
    env.set("not".to_string(), Value::NativeFunc(list_ops::not));
    env.set("car".to_string(), Value::NativeFunc(list_ops::car));
    env.set("first".to_string(), Value::NativeFunc(list_ops::car));
    env.set("cdr".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("rest".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("cons".to_string(), Value::NativeFunc(list_ops::cons));

    // Vector
    env.set("vector".to_string(), Value::NativeFunc(vector::vector));
    env.set("nth".to_string(), Value::NativeFunc(vector::nth));
    env.set("count".to_string(), Value::NativeFunc(vector::count));
    env.set("conj".to_string(), Value::NativeFunc(vector::conj));

    // Map
    env.set("hash-map".to_string(), Value::NativeFunc(map::hash_map));
    env.set("get".to_string(), Value::NativeFunc(map::get));
    env.set("assoc".to_string(), Value::NativeFunc(map::assoc));
    env.set("dissoc".to_string(), Value::NativeFunc(map::dissoc));
    env.set("keys".to_string(), Value::NativeFunc(map::keys));
    env.set("vals".to_string(), Value::NativeFunc(map::vals));
    env.set("contains?".to_string(), Value::NativeFunc(map::contains_q));

    // String operations
    env.set("str".to_string(), Value::NativeFunc(string_ops::str_fn));
    env.set(
        "string-length".to_string(),
        Value::NativeFunc(string_ops::string_length),
    );
    env.set(
        "substring".to_string(),
        Value::NativeFunc(string_ops::substring),
    );
    env.set(
        "string-append".to_string(),
        Value::NativeFunc(string_ops::string_append),
    );

    // Type predicates
    env.set("nil?".to_string(), Value::NativeFunc(type_ops::is_nil));
    env.set("list?".to_string(), Value::NativeFunc(type_ops::is_list));
    env.set(
        "number?".to_string(),
        Value::NativeFunc(type_ops::is_number),
    );
    env.set(
        "string?".to_string(),
        Value::NativeFunc(type_ops::is_string),
    );
    env.set(
        "symbol?".to_string(),
        Value::NativeFunc(type_ops::is_symbol),
    );
    env.set(
        "vector?".to_string(),
        Value::NativeFunc(type_ops::is_vector),
    );
    env.set("map?".to_string(), Value::NativeFunc(type_ops::is_map));
    env.set("type-of".to_string(), Value::NativeFunc(type_ops::type_of));
}
