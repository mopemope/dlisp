mod compile;
mod config;
mod repl;

use clap::{Parser, Subcommand};
use config::setup_logging;
use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, Interpreter};
use dlisp_core::parser::parse;
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optional script file to execute if no subcommand is given
    #[arg(required = false)]
    file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compile a script to a native executable
    Compile {
        /// Source file
        file: PathBuf,
        /// Output filename
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Enable optimization (speed)
        #[arg(short = 'O', long = "optimize")]
        optimize: bool,
        /// Release mode (speed_and_size optimization, verifier disabled)
        #[arg(long = "release")]
        release: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let _guard = setup_logging();

    let args = Cli::parse();

    if let Some(cmd) = args.command {
        match cmd {
            Commands::Compile {
                file,
                output,
                optimize,
                release,
            } => {
                compile::compile_file(file, output, optimize, release)?;
            }
        }
        return Ok(());
    }

    info!("Starting dlisp REPL...");
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

                        // Check for main entry point
                        let main_val = env.borrow().get("main");
                        if let Some(main_val) = main_val {
                            if let Value::UserFunc { .. } = main_val {
                                if let Err(e) = interpreter.apply(main_val, vec![], &mut env).await
                                {
                                    eprintln!("\x1b[31mError in main:\x1b[0m {}", e);
                                    std::process::exit(1);
                                }
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
                repl::run_repl().await
            }
        })
        .await
}

#[cfg(test)]
mod tests {

    use crate::config::get_history_path;
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
