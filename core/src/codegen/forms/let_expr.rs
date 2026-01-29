use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;
use std::collections::HashMap;

pub fn compile_let<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 3 {
        return Err("let requires bindings and body".to_string());
    }

    let bindings_clause = &list[1];
    let bindings = match bindings_clause {
        Value::List(l) => l,
        Value::Nil => &Vec::new()[..],
        _ => return Err("let bindings must be a list".to_string()),
    };

    // Evaluate bindings in CURRENT context
    let mut evaluated_bindings = Vec::new();
    for binding in bindings {
        if let Value::List(pair) = binding {
            if pair.len() != 2 {
                return Err("let binding invalid".to_string());
            }
            let name = match &pair[0] {
                Value::Symbol(s) => s.clone(),
                _ => return Err("let binding name must be symbol".to_string()),
            };
            let val = ctx.compile_expr(&pair[1])?;
            evaluated_bindings.push((name, val));
        } else {
            return Err("let binding must be a list".to_string());
        }
    }

    // Declare variables and push scope
    ctx.scopes.push(HashMap::new());
    {
        let current_scope = ctx.scopes.last_mut().unwrap();
        for (name, val) in evaluated_bindings {
            let var = ctx.builder.declare_var(ctx.ptr_type);
            ctx.builder.def_var(var, val);
            current_scope.insert(name, var);
        }
    }

    // Compile body
    let mut res = ctx.builder.ins().iconst(ctx.ptr_type, 0);
    for expr in &list[2..] {
        res = ctx.compile_expr(expr)?;
    }

    // Pop scope
    ctx.scopes.pop();

    Ok(res)
}
