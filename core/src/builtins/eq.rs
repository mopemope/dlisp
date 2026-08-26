use crate::ast::Value;

pub fn eq(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("= requires exactly 2 arguments".to_string());
        }

        fn is_equal(a: &Value, b: &Value) -> bool {
            match (a, b) {
                (Value::Integer(a), Value::Integer(b)) => a == b,
                (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
                (Value::Integer(a), Value::Float(b)) => (*a as f64 - *b).abs() < f64::EPSILON,
                (Value::Float(a), Value::Integer(b)) => (*a - *b as f64).abs() < f64::EPSILON,
                (Value::List(a), Value::List(b)) => {
                    if a.len() != b.len() {
                        return false;
                    }
                    a.iter().zip(b.iter()).all(|(x, y)| is_equal(x, y))
                }
                (Value::Vector(a), Value::Vector(b)) => {
                    if a.len() != b.len() {
                        return false;
                    }
                    a.iter().zip(b.iter()).all(|(x, y)| is_equal(x, y))
                }
                (Value::Map(a), Value::Map(b)) => {
                    if a.len() != b.len() {
                        return false;
                    }
                    // For Maps, we can't zip. We must check if every key in A exists in B and values are equal.
                    // Note: This relies on key equality being strict (Hash/Eq) for lookup.
                    // If we want loose key equality, it's O(N^2).
                    // We will stick to strict KEY equality, but loose VALUE equality.
                    for (k, v_a) in a {
                        if let Some(v_b) = b.get(k) {
                            if !is_equal(v_a, v_b) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                }
                (Value::Symbol(a), Value::Symbol(b)) => a == b,
                (Value::Keyword(a), Value::Keyword(b)) => a == b,
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Nil, Value::Nil) => true,
                (Value::Bool(a), Value::Bool(b)) => a == b,
                // Handles compare by identity (same underlying object).
                (Value::Channel(a), Value::Channel(b)) => a == b,
                (Value::Atom(a), Value::Atom(b)) => a == b,
                _ => false,
            }
        }

        Ok(Value::Bool(is_equal(&args[0], &args[1])))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eq_integers() {
        assert_eq!(
            eq(&[Value::Integer(1), Value::Integer(1)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            eq(&[Value::Integer(1), Value::Integer(2)]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_eq_floats() {
        assert_eq!(
            eq(&[Value::Float(1.0), Value::Float(1.0)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            eq(&[Value::Float(1.0), Value::Float(2.0)]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_eq_mixed() {
        assert_eq!(
            eq(&[Value::Integer(1), Value::Float(1.0)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            eq(&[Value::Float(1.0), Value::Integer(1)]).await,
            Ok(Value::Bool(true))
        );
    }

    #[tokio::test]
    async fn test_eq_other_types() {
        assert_eq!(
            eq(&[
                Value::Symbol("a".to_string()),
                Value::Symbol("a".to_string())
            ])
            .await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            eq(&[
                Value::String("a".to_string()),
                Value::String("a".to_string())
            ])
            .await,
            Ok(Value::Bool(true))
        );
        assert_eq!(eq(&[Value::Nil, Value::Nil]).await, Ok(Value::Bool(true)));
        assert_eq!(
            eq(&[Value::Nil, Value::Integer(0)]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_eq_keywords() {
        assert_eq!(
            eq(&[
                Value::Keyword("a".to_string()),
                Value::Keyword("a".to_string())
            ])
            .await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            eq(&[
                Value::Keyword("a".to_string()),
                Value::Keyword("b".to_string())
            ])
            .await,
            Ok(Value::Bool(false))
        );
        // Keyword != Symbol
        assert_eq!(
            eq(&[
                Value::Keyword("a".to_string()),
                Value::Symbol("a".to_string())
            ])
            .await,
            Ok(Value::Bool(false))
        );
    }
}
