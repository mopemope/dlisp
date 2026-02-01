use assert_cmd::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

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
