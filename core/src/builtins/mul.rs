use crate::ast::Value;

pub fn mul(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        let mut sum = 1;
        let mut float_sum = 1.0;
        let mut is_float = false;

        for arg in &args {
            match arg {
                Value::Integer(i) => {
                    if is_float {
                        float_sum *= *i as f64;
                    } else {
                        sum *= i;
                    }
                }
                Value::Float(f) => {
                    if !is_float {
                        is_float = true;
                        float_sum = sum as f64;
                    }
                    float_sum *= f;
                }
                _ => return Err("Arguments must be numbers".to_string()),
            }
        }

        if is_float {
            Ok(Value::Float(float_sum))
        } else {
            Ok(Value::Integer(sum))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mul_integers() {
        let args = vec![Value::Integer(2), Value::Integer(3), Value::Integer(4)];
        let result = mul(&args).await;
        assert_eq!(result, Ok(Value::Integer(24)));
    }

    #[tokio::test]
    async fn test_mul_floats() {
        let args = vec![Value::Float(2.0), Value::Float(3.0)];
        let result = mul(&args).await;
        assert_eq!(result, Ok(Value::Float(6.0)));
    }

    #[tokio::test]
    async fn test_mul_mixed() {
        let args = vec![Value::Integer(2), Value::Float(3.5)];
        let result = mul(&args).await;
        assert_eq!(result, Ok(Value::Float(7.0)));
    }
}
