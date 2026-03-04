use crate::ast::Value;
use std::io::Write;

pub fn print(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg);
        }
        println!();
        let _ = std::io::stdout().flush();
        Ok(Value::Nil)
    })
}

/// (println args...)
/// Prints all arguments separated by spaces, followed by a newline.
/// Identical to `print` but provided for naming clarity alongside `print-str`.
pub fn println_fn(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg);
        }
        println!();
        let _ = std::io::stdout().flush();
        Ok(Value::Nil)
    })
}

/// (print-str args...)
/// Prints all arguments separated by spaces WITHOUT a trailing newline.
/// Useful when building up output incrementally.
pub fn print_str_fn(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg);
        }
        let _ = std::io::stdout().flush();
        Ok(Value::Nil)
    })
}

/// (eprintln args...)
/// Prints all arguments separated by spaces to stderr, followed by a newline.
pub fn eprintln_fn(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                eprint!(" ");
            }
            eprint!("{}", arg);
        }
        eprintln!();
        let _ = std::io::stderr().flush();
        Ok(Value::Nil)
    })
}

/// (format fmt-string args...)
/// Returns a formatted string. Placeholders:
///   `{}` — replaced by the next argument's display representation
///   `{{` — literal `{`
///   `}}` — literal `}`
///
/// If there are more `{}` than remaining arguments, an error is returned.
/// Extra arguments beyond what the format string uses are ignored.
///
/// Examples:
///   (format "Hello, {}!" "world")      => "Hello, world!"
///   (format "{} + {} = {}" 1 2 3)      => "1 + 2 = 3"
///   (format "Use {{}} for braces")     => "Use {} for braces"
pub fn format_str(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() {
            return Err("format requires at least 1 argument (format string)".to_string());
        }

        let fmt = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("format first argument must be a string".to_string()),
        };

        let format_args = &args[1..];
        let mut result = String::new();
        let mut arg_idx = 0;
        let chars: Vec<char> = fmt.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if chars[i] == '{' {
                if i + 1 < len && chars[i + 1] == '{' {
                    // Escaped `{{` → literal `{`
                    result.push('{');
                    i += 2;
                } else if i + 1 < len && chars[i + 1] == '}' {
                    // `{}` → substitute next argument
                    if arg_idx >= format_args.len() {
                        return Err(format!(
                            "format: not enough arguments for placeholder #{} (have {} args)",
                            arg_idx + 1,
                            format_args.len()
                        ));
                    }
                    result.push_str(&format!("{}", format_args[arg_idx]));
                    arg_idx += 1;
                    i += 2;
                } else {
                    // Unmatched `{` — just output it literally
                    result.push('{');
                    i += 1;
                }
            } else if chars[i] == '}' {
                if i + 1 < len && chars[i + 1] == '}' {
                    // Escaped `}}` → literal `}`
                    result.push('}');
                    i += 2;
                } else {
                    // Unmatched `}` — just output it literally
                    result.push('}');
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        Ok(Value::String(result))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- print / println tests ---

    #[tokio::test]
    async fn test_print_returns_nil() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = print(&args).await;
        assert_eq!(result, Ok(Value::Nil));
    }

    #[tokio::test]
    async fn test_println_returns_nil() {
        let args = vec![Value::String("hello".to_string())];
        let result = println_fn(&args).await;
        assert_eq!(result, Ok(Value::Nil));
    }

    #[tokio::test]
    async fn test_print_str_returns_nil() {
        let args = vec![Value::Integer(42)];
        let result = print_str_fn(&args).await;
        assert_eq!(result, Ok(Value::Nil));
    }

    #[tokio::test]
    async fn test_eprintln_returns_nil() {
        let args = vec![Value::String("error msg".to_string())];
        let result = eprintln_fn(&args).await;
        assert_eq!(result, Ok(Value::Nil));
    }

    #[tokio::test]
    async fn test_print_no_args() {
        // (print) should just print a newline
        let result = print(&[]).await;
        assert_eq!(result, Ok(Value::Nil));
    }

    // --- format tests ---

    #[tokio::test]
    async fn test_format_basic() {
        let result = format_str(&[
            Value::String("Hello, {}!".to_string()),
            Value::String("world".to_string()),
        ])
        .await;
        assert_eq!(result, Ok(Value::String("Hello, world!".to_string())));
    }

    #[tokio::test]
    async fn test_format_multiple_placeholders() {
        let result = format_str(&[
            Value::String("{} + {} = {}".to_string()),
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
        .await;
        assert_eq!(result, Ok(Value::String("1 + 2 = 3".to_string())));
    }

    #[tokio::test]
    async fn test_format_no_placeholders() {
        let result = format_str(&[Value::String("no placeholders here".to_string())]).await;
        assert_eq!(
            result,
            Ok(Value::String("no placeholders here".to_string()))
        );
    }

    #[tokio::test]
    async fn test_format_escaped_braces() {
        let result = format_str(&[Value::String("Use {{}} for braces".to_string())]).await;
        assert_eq!(result, Ok(Value::String("Use {} for braces".to_string())));
    }

    #[tokio::test]
    async fn test_format_mixed_escape_and_placeholder() {
        let result = format_str(&[Value::String("{{{}}}".to_string()), Value::Integer(42)]).await;
        assert_eq!(result, Ok(Value::String("{42}".to_string())));
    }

    #[tokio::test]
    async fn test_format_not_enough_args() {
        let result = format_str(&[Value::String("{} and {}".to_string()), Value::Integer(1)]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("not enough arguments for placeholder")
        );
    }

    #[tokio::test]
    async fn test_format_extra_args_ignored() {
        let result = format_str(&[
            Value::String("only {}".to_string()),
            Value::Integer(1),
            Value::Integer(2), // extra, should be ignored
        ])
        .await;
        assert_eq!(result, Ok(Value::String("only 1".to_string())));
    }

    #[tokio::test]
    async fn test_format_no_args() {
        let result = format_str(&[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_format_non_string_fmt() {
        let result = format_str(&[Value::Integer(42)]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_format_with_various_types() {
        let result = format_str(&[
            Value::String("int:{} float:{} bool:{} nil:{}".to_string()),
            Value::Integer(42),
            Value::Float(3.5),
            Value::Bool(true),
            Value::Nil,
        ])
        .await;
        assert_eq!(
            result,
            Ok(Value::String(
                "int:42 float:3.5 bool:true nil:nil".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_format_unescaped_braces() {
        let result = format_str(&[Value::String("just a { and } here".to_string())]).await;
        assert_eq!(result, Ok(Value::String("just a { and } here".to_string())));
    }

    #[tokio::test]
    async fn test_format_empty_format_string() {
        let result = format_str(&[Value::String("".to_string())]).await;
        assert_eq!(result, Ok(Value::String("".to_string())));
    }

    #[tokio::test]
    async fn test_format_only_placeholders() {
        let result = format_str(&[
            Value::String("{}{}{}".to_string()),
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
        .await;
        assert_eq!(result, Ok(Value::String("abc".to_string())));
    }

    #[tokio::test]
    async fn test_format_unmatched_open_brace() {
        // A lone `{` without `}` should be output literally
        let result = format_str(&[Value::String("hello { world".to_string())]).await;
        assert_eq!(result, Ok(Value::String("hello { world".to_string())));
    }

    #[tokio::test]
    async fn test_format_unmatched_close_brace() {
        // A lone `}` without `{` should be output literally
        let result = format_str(&[Value::String("hello } world".to_string())]).await;
        assert_eq!(result, Ok(Value::String("hello } world".to_string())));
    }

    #[tokio::test]
    async fn test_format_with_list_value() {
        let result = format_str(&[
            Value::String("list: {}".to_string()),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        ])
        .await;
        assert_eq!(result, Ok(Value::String("list: (1 2 3)".to_string())));
    }

    #[tokio::test]
    async fn test_format_with_keyword() {
        let result = format_str(&[
            Value::String("key: {}".to_string()),
            Value::Keyword("name".to_string()),
        ])
        .await;
        assert_eq!(result, Ok(Value::String("key: :name".to_string())));
    }
}
