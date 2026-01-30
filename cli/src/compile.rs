use dlisp_core::compiler::{AOTCompiler, CompilerOptions, OptimizationLevel};
use dlisp_core::parser::parse;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub async fn compile_file(
    file: PathBuf,
    output: Option<PathBuf>,
    optimize: bool,
    release: bool,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(&file)?;
    match parse(&content) {
        Ok(vals) => {
            let mut options = CompilerOptions::default();
            if release {
                options.optimization_level = OptimizationLevel::SpeedAndSize;
                options.enable_verifier = false;
            } else if optimize {
                options.optimization_level = OptimizationLevel::Speed;
            }
            let compiler = AOTCompiler::with_options(options);
            match compiler.compile(vals).await {
                Ok(bytes) => {
                    let object_file = file.with_extension("o");
                    fs::write(&object_file, bytes)?;

                    let output_file = output
                        .or_else(|| file.file_stem().map(PathBuf::from))
                        .unwrap_or_else(|| PathBuf::from("a.out"));

                    info!("Linking object file {:?} to {:?}", object_file, output_file);

                    let mut lib_path = PathBuf::from("target/debug/libdlisp_runtime.a");
                    if !lib_path.exists() {
                        if let Ok(exe_path) = std::env::current_exe() {
                            let candidate = exe_path.parent().unwrap().join("libdlisp_runtime.a");
                            if candidate.exists() {
                                lib_path = candidate;
                            }
                        }
                    }

                    let status = Command::new("cc")
                        .arg("-no-pie")
                        .arg(&object_file)
                        .arg(&lib_path)
                        .arg("-lpthread")
                        .arg("-ldl")
                        .arg("-lm")
                        .arg("-lgc")
                        .arg("-o")
                        .arg(&output_file)
                        .status()?;

                    if !status.success() {
                        eprintln!("Linking failed");
                        std::process::exit(1);
                    }

                    // Cleanup object file
                    let _ = fs::remove_file(object_file);
                    println!("Compiled to {:?}", output_file);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Compilation Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("\x1b[31mParse Error:\x1b[0m {:?}", e);
            std::process::exit(1);
        }
    }
}
