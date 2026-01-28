use crate::ast::Value;

pub fn print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_success() {
        // (print 1 2) => nil
        // We can't easily capture stdout in unit tests without a crate like `subprocess` or redirecting,
        // but we can check the return value.
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = print(&args);
        assert_eq!(result, Ok(Value::Nil));
    }
}
