use crate::config::get_history_path;
use dlisp_core::interpreter::{default_env, Interpreter};
use dlisp_core::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub async fn run_repl() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let history_path = get_history_path();

    if let Some(ref path) = history_path {
        if rl.load_history(path).is_err() {
            // No previous history
        }
    }

    let env = default_env();
    let mut interpreter = Interpreter::new();

    println!("Welcome to dlisp v0.1.0");
    loop {
        let readline = rl.readline("🐕️> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                if line.trim().is_empty() {
                    continue;
                }

                match parse(&line) {
                    Ok(vals) => {
                        for val in vals {
                            match interpreter.eval(val, &mut env.clone()).await {
                                Ok(res) => println!("=> {}", res),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                    }
                    Err(errs) => {
                        for e in errs {
                            println!("Parse Error: {:?}", e);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    if let Some(ref path) = history_path {
        rl.save_history(path)?;
    }
    Ok(())
}
