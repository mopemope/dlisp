use clap::Parser;
use dlisp_core::interpreter::{default_env, Interpreter};
use dlisp_core::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    /// Optional script file to execute
    file: Option<PathBuf>,
}
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn get_state_dir() -> Option<PathBuf> {
    let mut path = dirs::state_dir().or_else(dirs::home_dir)?;
    path.push("dlisp");
    if fs::create_dir_all(&path).is_err() {
        return None;
    }
    Some(path)
}

fn get_history_path() -> Option<PathBuf> {
    let mut path = get_state_dir()?;
    path.push("history.txt");
    Some(path)
}

fn setup_logging() -> Option<WorkerGuard> {
    let log_dir = get_state_dir()?;
    let file_appender = tracing_appender::rolling::never(&log_dir, "debug.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    Some(guard)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let _guard = setup_logging();
    info!("Starting dlisp REPL...");

    let args = Cli::parse();
    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            if let Some(file) = args.file {
                let content = fs::read_to_string(file)?;
                let mut env = default_env();
                let mut interpreter = Interpreter::new();
                match parse(&content) {
                    Ok(vals) => {
                        for val in vals {
                            if let Err(e) = interpreter.eval(val, &mut env).await {
                                eprintln!("\x1b[31mError:\x1b[0m {}", e);
                                std::process::exit(1);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("\x1b[31mParse Error:\x1b[0m {:?}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                run_repl().await
            }
        })
        .await
}

async fn run_repl() -> anyhow::Result<()> {
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
