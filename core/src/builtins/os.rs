use crate::ast::Value;
use futures::future::LocalBoxFuture;
use std::collections::HashMap;
use tokio::process::Command;

pub fn sh(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() {
            return Err("sh requires at least 1 argument (command)".to_string());
        }

        let mut cmd_args = Vec::new();
        for arg in &args {
            if let Value::String(s) = arg {
                cmd_args.push(s.clone());
            } else {
                return Err("sh arguments must be strings".to_string());
            }
        }

        let mut command = if cmd_args.len() == 1 {
            // Run via sh -c if it's a single string
            let mut c = Command::new("sh");
            c.arg("-c").arg(&cmd_args[0]);
            c
        } else {
            // Explicit executable and arguments
            let mut c = Command::new(&cmd_args[0]);
            c.args(&cmd_args[1..]);
            c
        };

        match command.output().await {
            Ok(output) => {
                let mut map = HashMap::new();

                let status_code =
                    output
                        .status
                        .code()
                        .unwrap_or(if output.status.success() { 0 } else { -1 })
                        as i64;

                map.insert(
                    Value::Keyword("status".to_string()),
                    Value::Integer(status_code),
                );
                map.insert(
                    Value::Keyword("stdout".to_string()),
                    Value::String(String::from_utf8_lossy(&output.stdout).to_string()),
                );
                map.insert(
                    Value::Keyword("stderr".to_string()),
                    Value::String(String::from_utf8_lossy(&output.stderr).to_string()),
                );
                Ok(Value::Map(map))
            }
            // Instead of failing the Lisp evaluation entirely when the command is not found,
            // we could return status -1 and stderr with the error message.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut map = HashMap::new();
                map.insert(Value::Keyword("status".to_string()), Value::Integer(-1));
                map.insert(
                    Value::Keyword("stdout".to_string()),
                    Value::String("".to_string()),
                );
                map.insert(
                    Value::Keyword("stderr".to_string()),
                    Value::String(format!("Command not found: {}", e)),
                );
                Ok(Value::Map(map))
            }
            Err(e) => Err(format!("sh command execution failed: {}", e)),
        }
    })
}
