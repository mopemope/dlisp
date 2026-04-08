use assert_cmd::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn get_example_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // root
    path.push("example-lisp");
    path.push(name);
    path
}

fn dlisp_cmd() -> Command {
    cargo_bin_cmd!("dlisp")
}

fn write_temp_script(name: &str, content: &str) -> (TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("failed to create tempdir");
    let script = dir.path().join(format!("{}.lisp", name));
    let output = dir.path().join(name);
    fs::write(&script, content).expect("failed to write script");
    (dir, script, output)
}

#[test]
fn test_hello_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("hello.lisp");

    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, DLisp!"))
        .stdout(predicate::str::contains("15"));
}

#[test]
fn test_factorial_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("factorial.lisp");

    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("120"))
        .stdout(predicate::str::contains("3628800"));
}

#[test]
fn test_main_entry_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("main_entry.lisp");

    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Top level"))
        .stdout(predicate::str::contains("Hello from main!"));
}

#[test]
fn test_fib_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("fib.lisp");

    // fib 10 = 55
    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("55"));
}

#[test]
fn test_macros_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("macros.lisp");

    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("10 is greater than 5!"))
        .stdout(predicate::str::contains("1 is NOT equal to 2!"))
        .stdout(predicate::str::contains("100"))
        .stdout(predicate::str::contains("Recursive macro expansion works!"));
}

#[test]
fn test_spawn_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("spawn.lisp");

    cmd.arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1 finished"))
        .stdout(predicate::str::contains("Task 2 finished"))
        .stdout(predicate::str::contains("Main done."));
}

#[test]
fn test_error_lisp() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("error.lisp");

    cmd.arg(path).assert().failure().code(1);
}

#[test]
fn test_compile_error() {
    let mut cmd = dlisp_cmd();
    let path = get_example_path("error.lisp");

    // We expect compilation to fail because of undefined function or similar issues
    // caught during compilation phase if the compiler supports it, or maybe it succeeds
    // and runtime fails.
    // If the compiler is simple and allows any symbol as function call, it might succeed compilation.
    // But let's check `AOTCompiler` behavior or just try running it first.
    // Actually, let's verify if `dlisp compile` fails on `error.lisp` first manually.
    cmd.arg("compile").arg(path).assert().failure().code(1);
}

#[test]
fn test_syntax_error() {
    let path = get_example_path("syntax_error.lisp");

    // Interpreter
    let mut cmd = dlisp_cmd();
    cmd.arg(path.clone()).assert().failure().code(1);

    // Compiler
    let mut cmd = dlisp_cmd();
    cmd.arg("compile").arg(path).assert().failure().code(1);
}

#[test]
fn test_compile_executes_top_level_and_global_state() {
    let (_dir, script, output) = write_temp_script(
        "compiled_globals",
        r#"
(defvar greeting "hello from global")
(setq greeting "hello from setq")
(print "Top level init")

(defun main ()
  (print greeting))
"#,
    );

    let mut compile_cmd = dlisp_cmd();
    compile_cmd
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    Command::new(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Top level init"))
        .stdout(predicate::str::contains("hello from setq"));
}

#[test]
fn test_compile_supports_rest_let_star_destructure_and_collection_parity() {
    let (_dir, script, output) = write_temp_script(
        "compiled_surface_parity",
        r#"
(defun summarize (head &rest tail)
  (print head)
  (print tail))

(defun main ()
  (let* ((parts '(10 20 30))
         ((first &rest rest) parts)
         (picked (get {:a 1} :missing 99))
         (lst (conj '(1 2) 3 4)))
    (summarize first picked)
    (print rest)
    (print lst)))
"#,
    );

    let mut compile_cmd = dlisp_cmd();
    compile_cmd
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    Command::new(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("(99)"))
        .stdout(predicate::str::contains("(20 30)"))
        .stdout(predicate::str::contains("(4 3 1 2)"));
}
