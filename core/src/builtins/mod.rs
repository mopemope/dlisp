use crate::ast::Value;
use crate::environment::Environment;

pub mod add;
pub mod cmp;
pub mod div;
pub mod eq;
pub mod error_ops;
pub mod gt;
pub mod io;
pub mod list;
pub mod list_ops;
pub mod lt;
pub mod macro_ops;
pub mod map;
pub mod math;
pub mod mul;
pub mod os;
pub mod print;
pub mod sleep;
pub mod string_ops;
pub mod sub;
pub mod sys;
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
    env.set("max".to_string(), Value::NativeFunc(cmp::max));
    env.set("min".to_string(), Value::NativeFunc(cmp::min));
    env.set("abs".to_string(), Value::NativeFunc(math::abs));
    env.set("pow".to_string(), Value::NativeFunc(math::pow));

    // I/O
    env.set("print".to_string(), Value::NativeFunc(print::print));
    env.set("sleep".to_string(), Value::NativeFunc(sleep::sleep_fn));
    env.set("read-file".to_string(), Value::NativeFunc(io::read_file));
    env.set("write-file".to_string(), Value::NativeFunc(io::write_file));
    env.set(
        "file-exists?".to_string(),
        Value::NativeFunc(io::file_exists),
    );
    env.set("is-dir?".to_string(), Value::NativeFunc(io::is_dir));
    env.set("is-file?".to_string(), Value::NativeFunc(io::is_file));
    env.set(
        "delete-file".to_string(),
        Value::NativeFunc(io::delete_file),
    );
    env.set("list-dir".to_string(), Value::NativeFunc(io::list_dir));

    // Sys
    env.set("getenv".to_string(), Value::NativeFunc(sys::getenv));
    env.set("setenv".to_string(), Value::NativeFunc(sys::setenv));
    env.set("cwd".to_string(), Value::NativeFunc(sys::cwd));
    env.set("set-cwd".to_string(), Value::NativeFunc(sys::set_cwd));
    env.set("exit".to_string(), Value::NativeFunc(sys::exit));
    env.set("args".to_string(), Value::NativeFunc(sys::get_args));

    // OS
    env.set("sh".to_string(), Value::NativeFunc(os::sh));
    env.set("exec".to_string(), Value::NativeFunc(os::sh));

    // List
    env.set("list".to_string(), Value::NativeFunc(list::list));
    env.set("not".to_string(), Value::NativeFunc(list_ops::not));
    env.set("car".to_string(), Value::NativeFunc(list_ops::car));
    env.set("first".to_string(), Value::NativeFunc(list_ops::car));
    env.set("cdr".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("rest".to_string(), Value::NativeFunc(list_ops::cdr));
    env.set("cons".to_string(), Value::NativeFunc(list_ops::cons));
    env.set("append".to_string(), Value::NativeFunc(list_ops::append));
    env.set("reverse".to_string(), Value::NativeFunc(list_ops::reverse));
    env.set("sort".to_string(), Value::NativeFunc(list_ops::sort));

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

    // Macro utils
    env.set("gensym".to_string(), Value::NativeFunc(macro_ops::gensym));

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
    env.set(
        "string-split".to_string(),
        Value::NativeFunc(string_ops::string_split),
    );
    env.set(
        "string-replace".to_string(),
        Value::NativeFunc(string_ops::string_replace),
    );
    env.set(
        "string-upper".to_string(),
        Value::NativeFunc(string_ops::string_upper),
    );
    env.set(
        "string-lower".to_string(),
        Value::NativeFunc(string_ops::string_lower),
    );
    env.set(
        "string-trim".to_string(),
        Value::NativeFunc(string_ops::string_trim),
    );
    env.set(
        "string-trim-left".to_string(),
        Value::NativeFunc(string_ops::string_trim_left),
    );
    env.set(
        "string-trim-right".to_string(),
        Value::NativeFunc(string_ops::string_trim_right),
    );
    env.set(
        "string-starts-with?".to_string(),
        Value::NativeFunc(string_ops::string_starts_with),
    );
    env.set(
        "string-ends-with?".to_string(),
        Value::NativeFunc(string_ops::string_ends_with),
    );
    env.set(
        "string-contains?".to_string(),
        Value::NativeFunc(string_ops::string_contains),
    );
    env.set(
        "string-index-of".to_string(),
        Value::NativeFunc(string_ops::string_index_of),
    );
    env.set(
        "string->number".to_string(),
        Value::NativeFunc(string_ops::string_to_number),
    );
    env.set(
        "number->string".to_string(),
        Value::NativeFunc(string_ops::number_to_string),
    );
    env.set(
        "char-at".to_string(),
        Value::NativeFunc(string_ops::char_at),
    );

    // Type predicates
    env.set("nil?".to_string(), Value::NativeFunc(type_ops::is_nil));
    env.set("error?".to_string(), Value::NativeFunc(type_ops::is_error));
    env.set("empty?".to_string(), Value::NativeFunc(type_ops::is_empty));
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
        "keyword?".to_string(),
        Value::NativeFunc(type_ops::is_keyword),
    );
    env.set(
        "vector?".to_string(),
        Value::NativeFunc(type_ops::is_vector),
    );
    env.set("map?".to_string(), Value::NativeFunc(type_ops::is_map));
    env.set("type-of".to_string(), Value::NativeFunc(type_ops::type_of));

    // Errors
    env.set(
        "error-value".to_string(),
        Value::NativeFunc(error_ops::error_value),
    );
}
