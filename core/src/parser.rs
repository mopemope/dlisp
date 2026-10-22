use crate::ast::Value;
use chumsky::prelude::*;

/// Maximum allowed expression nesting depth accepted by [`parse`].
///
/// Inputs deeper than this are rejected with a tidy parse error before the
/// recursive descent parser (or the evaluator) can overflow the stack.
/// Measured crash points put the parser above this limit by a wide margin.
pub const MAX_EXPRESSION_NESTING: usize = 4096;

/// The parser's symbol-character set, mirrored from the `symbol_char`
/// combinator below so scanning skips the same token shapes.
fn is_symbol_char(c: char) -> bool {
    c.is_alphanumeric() || "+-*/!@$%^&_=<>?:".contains(c)
}

enum ScanError {
    /// Byte offset of the opener that would exceed the nesting limit.
    TooDeep(usize),
    /// Byte offset of the start of an out-of-range integer literal.
    IntOverflow(usize),
}

/// Single left-to-right pass over the source, ahead of the real parser.
///
/// Tracks bracket nesting (skipping string contents and `;` comments) and
/// detects integer literals whose digit runs do not fit in `i64`. Symbol
/// runs (e.g. `my-123456…`) are skipped as a whole so only true numeric
/// literals are range-checked.
fn scan_source(src: &str) -> Result<(), ScanError> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut in_comment = false;
    let mut chars = src.char_indices().peekable();

    while let Some((start, c)) = chars.next() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            continue;
        }
        if in_string {
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            ';' => in_comment = true,
            '"' => in_string = true,
            '(' | '[' | '{' => {
                if depth == MAX_EXPRESSION_NESTING {
                    return Err(ScanError::TooDeep(start));
                }
                depth += 1;
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
            }
            c if is_symbol_char(c) => {
                // Consume the whole symbol-style run, then range-check it
                // only when it is exactly an optionally-signed digit run.
                let mut end = start + c.len_utf8();
                while let Some(&(i, next)) = chars.peek() {
                    if !is_symbol_char(next) {
                        break;
                    }
                    end = i + next.len_utf8();
                    chars.next();
                }
                let token = &src[start..end];
                let digits = token.strip_prefix('-').unwrap_or(token);
                if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
                    // Parse with the sign kept so i64::MIN stays valid.
                    if token.parse::<i64>().is_err() {
                        return Err(ScanError::IntOverflow(start));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn parser<'src>() -> impl Parser<'src, &'src str, Vec<Value>, extra::Err<Rich<'src, char>>> {
    let float = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.'))
        .then(text::int(10))
        .map(|(((sign, int_part), _dot), frac_part)| {
            let mut s = String::new();
            if sign.is_some() {
                s.push('-');
            }
            s.push_str(int_part);
            s.push('.');
            s.push_str(frac_part);
            Value::Float(s.parse().unwrap())
        });

    let int = just('-')
        .or_not()
        .then(text::int(10))
        .try_map(|(sign, s), span| {
            let mut res = String::new();
            if sign.is_some() {
                res.push('-');
            }
            res.push_str(s);
            res.parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| Rich::custom(span, "integer literal out of range"))
        });
    // Keywords nil/true/false must not be followed by symbol characters,
    // otherwise "nil?" should parse as a single symbol.
    let not_symbol_next =
        none_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-*/!@$%^&_=<>?:")
            .rewind()
            .ignored()
            .or(end().to(()));

    let boolean = just("true")
        .then_ignore(not_symbol_next)
        .to(Value::Bool(true))
        .or(just("false")
            .then_ignore(not_symbol_next)
            .to(Value::Bool(false)));

    // Lisp symbols: letters, digits, and extended characters
    let symbol_char =
        any().filter(|c: &char| c.is_alphanumeric() || "+-*/!@$%^&_=<>?:".contains(*c));

    let symbol = symbol_char
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Value::Symbol);

    let keyword = just(':')
        .ignore_then(symbol_char.repeated().at_least(1).collect::<String>())
        .map(Value::Keyword);

    let string = just('"')
        .ignore_then(any().filter(|c| *c != '"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(Value::String);

    let nil = just("nil").then_ignore(not_symbol_next).to(Value::Nil);

    let comment = just(';')
        .ignore_then(any().filter(|c| *c != '\n').repeated())
        .padded();

    recursive(|expr| {
        let list = expr
            .clone()
            .padded()
            .repeated()
            .collect()
            .delimited_by(just('('), just(')'))
            .map(Value::List);

        let vector = expr
            .clone()
            .padded()
            .repeated()
            .collect()
            .delimited_by(just('['), just(']'))
            .map(Value::Vector);

        let map = expr
            .clone()
            .padded()
            .repeated()
            .collect::<Vec<Value>>()
            .delimited_by(just('{'), just('}'))
            .try_map(|vec, span| {
                if vec.len() % 2 != 0 {
                    Err(Rich::custom(
                        span,
                        "Map literal must have even number of elements",
                    ))
                } else {
                    let mut m = std::collections::HashMap::new();
                    for chunk in vec.chunks(2) {
                        m.insert(chunk[0].clone(), chunk[1].clone());
                    }
                    Ok(Value::Map(m))
                }
            });

        let quoted = just('\'')
            .ignore_then(expr.clone())
            .map(|v| Value::List(vec![Value::Symbol("quote".to_string()), v]));

        let backquoted = just('`')
            .ignore_then(expr.clone())
            .map(|v| Value::List(vec![Value::Symbol("backquote".to_string()), v]));

        let unquote_spliced = just(",@")
            .ignore_then(expr.clone())
            .map(|v| Value::List(vec![Value::Symbol("unquote-splicing".to_string()), v]));

        let unquoted = just(',')
            .ignore_then(expr.clone())
            .map(|v| Value::List(vec![Value::Symbol("unquote".to_string()), v]));

        // Order matters: float before int, boolean/nil before symbol if strictly overlapping,
        // but here true/false/nil are specific keywords.
        choice((
            float,
            int,
            boolean,
            nil,
            string,
            list,
            vector,
            map,
            quoted,
            backquoted,
            unquote_spliced,
            unquoted,
            keyword, // keyword before symbol since both start with alphanumeric but keyword has :
            symbol,  // symbol is last catch-all for identifiers
        ))
        .padded_by(comment.repeated()) // parsing comments trailing/surrounding values
    })
    .padded_by(comment.repeated())
    .padded()
    .repeated()
    .collect()
    .then_ignore(end())
}

pub fn parse(src: &str) -> Result<Vec<Value>, Vec<Rich<'_, char>>> {
    if let Err(err) = scan_source(src) {
        let (pos, msg) = match err {
            ScanError::TooDeep(pos) => (
                pos,
                format!(
                    "expression nesting too deep (exceeds limit {})",
                    MAX_EXPRESSION_NESTING
                ),
            ),
            ScanError::IntOverflow(pos) => (pos, "integer literal out of range".to_string()),
        };
        return Err(vec![Rich::custom(SimpleSpan::from(pos..pos + 1), msg)]);
    }
    parser().parse(src).into_result()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basics() {
        assert_eq!(parse("123").unwrap()[0], Value::Integer(123));
        assert_eq!(parse("12.3").unwrap()[0], Value::Float(12.3));
        assert_eq!(parse("true").unwrap()[0], Value::Bool(true));
        assert_eq!(parse("nil").unwrap()[0], Value::Nil);
        assert_eq!(parse("foo").unwrap()[0], Value::Symbol("foo".to_string()));
        assert_eq!(
            parse("\"bar\"").unwrap()[0],
            Value::String("bar".to_string())
        );
    }

    #[test]
    fn test_parse_list() {
        // Test with extra whitespace
        let vals = parse("(  1   2 )").unwrap();
        let val = &vals[0];
        if let Value::List(v) = val {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Value::Integer(1));
            assert_eq!(v[1], Value::Integer(2));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_nested() {
        let vals = parse("(+ 1 (* 2 3))").unwrap();
        let val = &vals[0];
        if let Value::List(v) = val {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Symbol("+".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_vector() {
        let vals = parse("[1 2 3]").unwrap();
        let val = &vals[0];
        if let Value::Vector(v) = val {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Integer(1));
            assert_eq!(v[1], Value::Integer(2));
            assert_eq!(v[2], Value::Integer(3));
        } else {
            panic!("Expected vector");
        }
    }

    #[test]
    fn test_parse_nested_vector() {
        let vals = parse("[1 (2 3) [4 5]]").unwrap();
        let val = &vals[0];
        if let Value::Vector(v) = val {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Integer(1));
            if let Value::List(l) = &v[1] {
                assert_eq!(l.len(), 2);
            } else {
                panic!("Expected list nested in vector");
            }
            if let Value::Vector(vec) = &v[2] {
                assert_eq!(vec.len(), 2);
            } else {
                panic!("Expected vector nested in vector");
            }
        } else {
            panic!("Expected vector");
        }
    }

    #[test]
    fn test_parse_map() {
        let vals = parse("{:a 1 :b 2}").unwrap();
        let val = &vals[0];
        if let Value::Map(m) = val {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_parse_map_nested() {
        let vals = parse("{:a {1 2}}").unwrap();
        let val = &vals[0];
        if let Value::Map(m) = val {
            assert_eq!(m.len(), 1);
            let inner = m.get(&Value::Keyword("a".to_string())).unwrap();
            if let Value::Map(m2) = inner {
                assert_eq!(m2.len(), 1);
                assert_eq!(m2.get(&Value::Integer(1)), Some(&Value::Integer(2)));
            } else {
                panic!("Expected inner map");
            }
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_parse_map_odd_elements() {
        let res = parse("{:a 1 :b}");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_integer_literal_overflow_is_error() {
        // An i64-overflowing integer literal must become a parse error,
        // never an unwrap panic (regression for echo 999...9 | dlisp file).
        assert!(parse("99999999999999999999999").is_err());
        assert!(parse("-99999999999999999999999").is_err());
        // i64 boundary values must keep parsing successfully.
        assert_eq!(
            parse("9223372036854775807").unwrap()[0],
            Value::Integer(i64::MAX)
        );
        assert_eq!(
            parse("-9223372036854775808").unwrap()[0],
            Value::Integer(i64::MIN)
        );
    }

    #[test]
    fn test_parse_excessive_nesting_is_error() {
        // Nesting past MAX_EXPRESSION_NESTING must be rejected with a tidy
        // parse error instead of exhausting the stack (a 65k-deep tree
        // crashed the CLI before the pre-scan existed).
        let src = format!("{}1{}", "(".repeat(10_000), ")".repeat(10_000));
        assert!(parse(&src).is_err());
    }

    #[test]
    fn test_parse_nesting_at_limit_still_parses() {
        // Depth exactly equal to MAX_EXPRESSION_NESTING must keep parsing.
        // Built and dropped in a big-stack thread: the deeply recursive
        // parser/Value drop needs far more than a 2 MiB test thread.
        let handle = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let src = format!(
                    "{}1{}",
                    "(".repeat(MAX_EXPRESSION_NESTING),
                    ")".repeat(MAX_EXPRESSION_NESTING)
                );
                assert!(parse(&src).is_ok());
            })
            .unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_parse_backquote_unquote() {
        let vals = parse("`a").unwrap();
        assert_eq!(
            vals[0],
            Value::List(vec![
                Value::Symbol("backquote".to_string()),
                Value::Symbol("a".to_string())
            ])
        );

        let vals = parse(",a").unwrap();
        assert_eq!(
            vals[0],
            Value::List(vec![
                Value::Symbol("unquote".to_string()),
                Value::Symbol("a".to_string())
            ])
        );

        let vals = parse(",@a").unwrap();
        assert_eq!(
            vals[0],
            Value::List(vec![
                Value::Symbol("unquote-splicing".to_string()),
                Value::Symbol("a".to_string())
            ])
        );

        let vals = parse("`(1 ,2 ,@3)").unwrap();
        if let Value::List(v) = &vals[0] {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Value::Symbol("backquote".to_string()));
            if let Value::List(inner) = &v[1] {
                assert_eq!(inner.len(), 3);
                assert_eq!(inner[0], Value::Integer(1));
                assert_eq!(
                    inner[1],
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Integer(2)
                    ])
                );
                assert_eq!(
                    inner[2],
                    Value::List(vec![
                        Value::Symbol("unquote-splicing".to_string()),
                        Value::Integer(3)
                    ])
                );
            } else {
                panic!("Expected list for backquoted form");
            }
        } else {
            panic!("Expected list for backquote");
        }
    }
}
