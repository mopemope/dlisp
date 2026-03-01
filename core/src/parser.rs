use crate::ast::Value;
use chumsky::prelude::*;

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

    let int = just('-').or_not().then(text::int(10)).map(|(sign, s)| {
        let mut res = String::new();
        if sign.is_some() {
            res.push('-');
        }
        res.push_str(s);
        Value::Integer(res.parse().unwrap())
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
            .ignore_then(expr)
            .map(|v| Value::List(vec![Value::Symbol("quote".to_string()), v]));

        // Order matters: float before int, boolean/nil before symbol if strictly overlapping,
        // but here true/false/nil are specific keywords.
        choice((
            float, int, boolean, nil, string, list, vector, map, quoted,
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
}
