use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;
use std::collections::HashMap;

pub fn compile_let<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
    sequential: bool,
) -> Result<IrValue, String> {
    if list.len() < 3 {
        return Err(if sequential {
            "let* requires bindings and body".to_string()
        } else {
            "let requires bindings and body".to_string()
        });
    }

    let bindings_clause = &list[1];
    let bindings = match bindings_clause {
        Value::List(l) => l,
        Value::Nil => &Vec::new()[..],
        _ => {
            return Err(if sequential {
                "let* bindings must be a list".to_string()
            } else {
                "let bindings must be a list".to_string()
            });
        }
    };

    ctx.scopes.push(HashMap::new());

    if sequential {
        for binding in bindings {
            if let Value::List(pair) = binding {
                if pair.len() != 2 {
                    return Err("let* binding invalid".to_string());
                }
                let val = ctx.compile_expr(&pair[1])?;
                bind_pattern(ctx, &pair[0], val)?;
            } else {
                return Err("let* binding must be a list".to_string());
            }
        }
    } else {
        let mut evaluated_bindings = Vec::new();
        for binding in bindings {
            if let Value::List(pair) = binding {
                if pair.len() != 2 {
                    return Err("let binding invalid".to_string());
                }
                let val = ctx.compile_expr(&pair[1])?;
                evaluated_bindings.push((&pair[0], val));
            } else {
                return Err("let binding must be a list".to_string());
            }
        }

        for (pattern, val) in evaluated_bindings {
            bind_pattern(ctx, pattern, val)?;
        }
    }

    let mut res = ctx.make_nil()?;
    for expr in &list[2..] {
        res = ctx.compile_expr(expr)?;
    }

    ctx.scopes.pop();

    Ok(res)
}

fn bind_pattern<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    pattern: &Value,
    val: IrValue,
) -> Result<(), String> {
    match pattern {
        Value::Symbol(name) => {
            let var = ctx.builder.declare_var(ctx.ptr_type);
            ctx.builder.def_var(var, val);
            ctx.scopes
                .last_mut()
                .expect("let scope exists")
                .insert(name.clone(), var);
            Ok(())
        }
        Value::List(items) => {
            let mut current = val;
            let mut iter = items.iter().peekable();
            while let Some(item) = iter.next() {
                if let Value::Symbol(s) = item
                    && s == "&rest"
                {
                    let rest_pat = iter
                        .next()
                        .ok_or_else(|| "&rest must be followed by a variable".to_string())?;
                    if iter.next().is_some() {
                        return Err(
                            "&rest must be the last element in destructuring pattern".to_string()
                        );
                    }
                    return bind_pattern(ctx, rest_pat, current);
                }

                let car_func = ctx
                    .module
                    .declare_func_in_func(ctx.builtins.funcs.dlisp_car, ctx.builder.func);
                let car_call = ctx.builder.ins().call(car_func, &[current]);
                let car_val = ctx.builder.inst_results(car_call)[0];
                bind_pattern(ctx, item, car_val)?;

                let cdr_func = ctx
                    .module
                    .declare_func_in_func(ctx.builtins.funcs.dlisp_cdr, ctx.builder.func);
                let cdr_call = ctx.builder.ins().call(cdr_func, &[current]);
                current = ctx.builder.inst_results(cdr_call)[0];
            }
            Ok(())
        }
        _ => Err("invalid destructuring pattern".to_string()),
    }
}
