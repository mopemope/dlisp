use crate::ast::Value;
use chumsky::prelude::*;

pub fn parser<'src>() -> impl Parser<'src, &'src str, Vec<Value>, extra::Err<Simple<'src, char>>> {
    let float = text::int(10)
        .then(just('.'))
        .then(text::int(10))
        .map(|((int_part, _dot), frac_part)| format!("{}.{}", int_part, frac_part))
        .map(|s: String| Value::Float(s.parse().unwrap()));

    let int = text::int(10).map(|s: &str| Value::Integer(s.parse().unwrap()));

    let boolean = just("true")
        .to(Value::Bool(true))
        .or(just("false").to(Value::Bool(false)));

    // Lisp symbols: letters, digits, and extended characters
    let symbol_char =
        any().filter(|c: &char| c.is_alphanumeric() || "+-*/!@$%^&_=<>?".contains(*c));

    let symbol = symbol_char
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Value::Symbol);

    let string = just('"')
        .ignore_then(any().filter(|c| *c != '"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(Value::String);

    let nil = just("nil").to(Value::Nil);

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

        let quoted = just('\'')
            .ignore_then(expr)
            .map(|v| Value::List(vec![Value::Symbol("quote".to_string()), v]));

        // Order matters: float before int, boolean/nil before symbol if strictly overlapping,
        // but here true/false/nil are specific symbols technically.
        // We put specific keywords first.
        choice((
            float, int, boolean, nil, string, list, quoted,
            symbol, // symbol is last catch-all for identifiers
        ))
        .padded_by(comment.repeated()) // parsing comments trailing/surrounding values
    })
    .padded_by(comment.repeated())
    .padded()
    .repeated()
    .collect()
    .then_ignore(end())
}

pub fn parse(src: &str) -> Result<Vec<Value>, Vec<Simple<'_, char>>> {
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
}
