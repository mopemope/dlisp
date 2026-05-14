use dlisp_core::compiler::{AOTCompiler, CompilerOptions, OptimizationLevel};
use dlisp_core::parser::parse;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

fn find_runtime_staticlib(release: bool) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let profile = if release { "release" } else { "debug" };
    candidates.push(PathBuf::from(format!(
        "target/{}/libdlisp_runtime.a",
        profile
    )));
    candidates.push(PathBuf::from(format!("target/{}/deps", profile)));

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("libdlisp_runtime.a"));
            candidates.push(exe_dir.join("deps"));
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("libdlisp_runtime.a"));
                candidates.push(parent.join("deps"));
            }
        }
    }

    for candidate in candidates {
        if candidate.is_file()
            && candidate
                .file_name()
                .is_some_and(|n| n == "libdlisp_runtime.a")
        {
            return Some(candidate);
        }
        if candidate.is_dir() {
            if let Ok(entries) = fs::read_dir(&candidate) {
                let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();
                    if name.starts_with("libdlisp_runtime-") && name.ends_with(".a") {
                        let modified = entry
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        let should_replace = match &best {
                            Some((current, _)) => modified > *current,
                            None => true,
                        };
                        if should_replace {
                            best = Some((modified, path));
                        }
                    }
                }
                if let Some((_, path)) = best {
                    return Some(path);
                }
            }
        }
    }

    None
}

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
            let base_dir = file.parent().unwrap_or_else(|| Path::new("."));
            match compiler.compile_with_base_dir(vals, base_dir).await {
                Ok(bytes) => {
                    let object_file = file.with_extension("o");
                    fs::write(&object_file, bytes)?;

                    let output_file = output
                        .or_else(|| file.file_stem().map(PathBuf::from))
                        .unwrap_or_else(|| PathBuf::from("a.out"));

                    info!("Linking object file {:?} to {:?}", object_file, output_file);

                    let lib_path = find_runtime_staticlib(release).ok_or_else(|| {
                        anyhow::anyhow!("failed to locate libdlisp_runtime static library")
                    })?;

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
