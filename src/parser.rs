use crate::ast::Value;
use chumsky::prelude::*;

pub fn parser() -> impl Parser<char, Value, Error = Simple<char>> {
    let float = text::digits(10)
        .then(just('.'))
        .then(text::digits(10))
        .map(|((int_part, _dot), frac_part)| format!("{}.{}", int_part, frac_part))
        .map(|s: String| Value::Float(s.parse().unwrap()));

    let int = text::int(10).map(|s: String| Value::Integer(s.parse().unwrap()));

    let boolean = just("true")
        .to(Value::Bool(true))
        .or(just("false").to(Value::Bool(false)));

    // Lisp symbols: letters, digits, and extended characters
    let symbol_char = filter(|c: &char| c.is_alphanumeric() || "+-*/!@$%^&_=<>?".contains(*c));

    let symbol = symbol_char
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Value::Symbol);

    let string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Value::String);

    let nil = just("nil").to(Value::Nil);

    recursive(|expr| {
        let list = expr
            .padded()
            .repeated()
            .delimited_by(just('('), just(')'))
            .map(Value::List);

        // Order matters: float before int, boolean/nil before symbol if strictly overlapping,
        // but here true/false/nil are specific symbols technically.
        // We put specific keywords first.
        choice((
            float, int, boolean, nil, string, list,
            symbol, // symbol is last catch-all for identifiers
        ))
    })
    .padded()
    .then_ignore(end())
}

pub fn parse(src: &str) -> Result<Value, Vec<Simple<char>>> {
    parser().parse(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basics() {
        assert_eq!(parse("123").unwrap(), Value::Integer(123));
        assert_eq!(parse("12.3").unwrap(), Value::Float(12.3));
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
        assert_eq!(parse("nil").unwrap(), Value::Nil);
        assert_eq!(parse("foo").unwrap(), Value::Symbol("foo".to_string()));
        assert_eq!(parse("\"bar\"").unwrap(), Value::String("bar".to_string()));
    }

    #[test]
    fn test_parse_list() {
        // Test with extra whitespace
        let val = parse("(  1   2 )").unwrap();
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
        let val = parse("(+ 1 (* 2 3))").unwrap();
        if let Value::List(v) = val {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Symbol("+".to_string()));
        } else {
            panic!("Expected list");
        }
    }
}
