//! Parity golden tests: every bundled example must produce equivalent output
//! under the interpreter/JIT path and the AOT-compiled path.
//!
//! Examples excluded here are covered elsewhere:
//! - `error.lisp` / `syntax_error.lisp`: expected-failure fixtures
//!   (`test_compile_error`, parse error tests).
//!
//! `spawn.lisp` is compared order-insensitively: spawned tasks map to OS
//! threads in compiled code, so concurrent stdout writes may interleave
//! between the text and its newline. Line content itself must still match.

use assert_cmd::cargo_bin_cmd;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const GOLDEN_EXAMPLES: &[&str] = &[
    "closure_indirect_test.lisp",
    "factorial.lisp",
    "fib.lisp",
    "hello.lisp",
    "higher_order_aot_test.lisp",
    "keyword_aot_test.lisp",
    "macros.lisp",
    "main_entry.lisp",
    "map_aot_test.lisp",
    "mutual_recursion.lisp",
    "no_main.lisp",
    "stdlib_range.lisp",
    "vector.lisp",
];

const CONCURRENT_EXAMPLES: &[&str] = &["spawn.lisp"];

fn example_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // root
    path.push("example-lisp");
    path
}

struct RunResult {
    stdout: String,
    code: i32,
}

fn run_interpreted(path: &Path) -> RunResult {
    let out = cargo_bin_cmd!("dlisp")
        .arg(path)
        .output()
        .expect("failed to run dlisp interpreter");
    RunResult {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

fn run_compiled(source: &Path, dir: &TempDir, name: &str) -> RunResult {
    // Copy the source into a temp dir so compile artifacts (*.o) never
    // litter example-lisp/, and parallel tests cannot collide.
    let local_source = dir.path().join(name);
    fs::copy(source, &local_source).expect("failed to copy example");

    let binary = dir.path().join("prog");
    let compile = cargo_bin_cmd!("dlisp")
        .arg("compile")
        .arg(&local_source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("failed to run dlisp compile");
    assert!(
        compile.status.success(),
        "AOT compile failed for {}: {}",
        name,
        String::from_utf8_lossy(&compile.stderr)
    );

    let out = Command::new(&binary)
        .current_dir(dir.path())
        .output()
        .expect("failed to run compiled program");
    RunResult {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

fn line_set(stdout: &str) -> BTreeSet<String> {
    stdout
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn assert_parity(name: &str, interpreted: &RunResult, compiled: &RunResult) {
    if CONCURRENT_EXAMPLES.contains(&name) {
        assert_eq!(
            line_set(&interpreted.stdout),
            line_set(&compiled.stdout),
            "output lines diverge between interpreter and AOT for {name}"
        );
    } else {
        assert_eq!(
            interpreted.stdout, compiled.stdout,
            "stdout diverges between interpreter and AOT for {name}"
        );
    }
    assert_eq!(
        interpreted.code, compiled.code,
        "exit status diverges between interpreter and AOT for {name}"
    );
}

#[test]
fn examples_match_interpreter_and_aot_output() {
    let dir = example_dir();
    assert!(
        dir.is_dir(),
        "example-lisp directory not found at {}",
        dir.display()
    );

    for name in GOLDEN_EXAMPLES
        .iter()
        .copied()
        .chain(CONCURRENT_EXAMPLES.iter().copied())
    {
        let source = dir.join(name);
        assert!(source.exists(), "missing example: {}", name);

        let interpreted = run_interpreted(&source);
        let tempdir = tempfile::tempdir().expect("failed to create tempdir");
        let compiled = run_compiled(&source, &tempdir, name);
        assert_parity(name, &interpreted, &compiled);
    }
}
