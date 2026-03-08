use crate::ast::Value;

pub fn bind_destructure(
    pattern: &Value,
    val: &Value,
    bindings: &mut Vec<(String, Value)>,
) -> Result<(), String> {
    match pattern {
        Value::Symbol(s) => {
            bindings.push((s.clone(), val.clone()));
            Ok(())
        }
        Value::List(pat_list) => {
            let val_list = match val {
                Value::List(l) => l,
                _ => return Err("destructuring bind expects a list value".to_string()),
            };

            let nil_val = Value::Nil;
            let mut i = 0;
            let mut pat_iter = pat_list.iter().peekable();
            while let Some(p) = pat_iter.next() {
                if let Value::Symbol(s) = p
                    && s == "&rest" {
                        if let Some(rest_pat) = pat_iter.next() {
                            if pat_iter.next().is_some() {
                                return Err(
                                    "&rest must be the last element in destructuring pattern"
                                        .to_string(),
                                );
                            }
                            let rest_val = if i < val_list.len() {
                                Value::List(val_list[i..].to_vec())
                            } else {
                                Value::Nil
                            };
                            return bind_destructure(rest_pat, &rest_val, bindings);
                        } else {
                            return Err("&rest must be followed by a variable".to_string());
                        }
                    }

                let v = if i < val_list.len() {
                    &val_list[i]
                } else {
                    &nil_val
                };
                bind_destructure(p, v, bindings)?;
                i += 1;
            }
            Ok(())
        }
        _ => Err("invalid destructuring pattern".to_string()),
    }
}
