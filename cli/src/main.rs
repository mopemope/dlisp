use dlisp_core::interpreter::{default_env, Interpreter};
use dlisp_core::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::path::PathBuf;

fn get_history_path() -> Option<PathBuf> {
    let mut path = dirs::state_dir().or_else(|| dirs::home_dir())?;
    path.push("dlisp");
    if let Err(_) = fs::create_dir_all(&path) {
        return None;
    }
    path.push("history.txt");
    Some(path)
}

fn main() -> anyhow::Result<()> {
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
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                if line.trim().is_empty() {
                    continue;
                }

                match parse(&line) {
                    Ok(ast) => match interpreter.eval(ast, &mut env.clone()) {
                        Ok(val) => println!("=> {}", val),
                        Err(e) => println!("Error: {}", e),
                    },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_history_path_xdg() {
        // Create a temporary directory to act as XDG_STATE_HOME
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Set XDG_STATE_HOME to the temp directory
        // unsafe block is needed because setting env vars in multi-threaded tests is unsafe
        // distinct test scope is required
        unsafe {
            env::set_var("XDG_STATE_HOME", &temp_path);
        }

        let history_path = get_history_path();
        
        // Cleanup env var just in case (though test isolation makes this tricky in parallel)
        unsafe {
            env::remove_var("XDG_STATE_HOME");
        }

        assert!(history_path.is_some());
        let path = history_path.unwrap();
        
        // expected path: $XDG_STATE_HOME/dlisp/history.txt
        let expected = temp_path.join("dlisp").join("history.txt");
        assert_eq!(path, expected);

        // Verify directory was created
        assert!(temp_path.join("dlisp").exists());
        assert!(temp_path.join("dlisp").is_dir());
    }
}
