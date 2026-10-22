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
        save_history_quietly(&mut rl, path);
    }
    Ok(())
}

/// Best-effort history save: a save failure must not turn a clean REPL
/// exit (CTRL-C / CTRL-D) into an error exit; warn and keep exit 0.
fn save_history_quietly(rl: &mut DefaultEditor, path: &std::path::Path) -> bool {
    if let Err(err) = rl.save_history(path) {
        eprintln!(
            "Warning: could not save REPL history to {}: {}",
            path.display(),
            err
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_history_quietly_reports_failure_without_panicking() {
        let mut rl = DefaultEditor::new().expect("editor");
        // An empty history is a no-op success in rustyline; memory one
        // entry so the save actually writes.
        rl.add_history_entry("(+ 1 2)").expect("history entry");
        let temp = tempfile::tempdir().expect("tempdir");
        // A directory named history.txt makes the save fail cleanly
        // (rustyline creates missing parent dirs, so removal games won't
        // force an error — point at an unopenable path instead).
        std::fs::create_dir(temp.path().join("history.txt")).expect("poison path");

        assert!(!save_history_quietly(
            &mut rl,
            &temp.path().join("history.txt")
        ));
    }

    #[test]
    fn test_save_history_quietly_succeeds_on_writable_path() {
        let mut rl = DefaultEditor::new().expect("editor");
        rl.add_history_entry("(+ 1 2)").expect("history entry");
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("history.txt");

        assert!(save_history_quietly(&mut rl, &path));
    }
}
