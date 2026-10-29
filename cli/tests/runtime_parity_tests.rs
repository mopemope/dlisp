//! Execute runtime regressions in child processes: a broken FFI must fail
//! an assertion here rather than aborting the test harness.
use assert_cmd::{cargo_bin_cmd, Command};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

fn run_case(body: &str, expected: &str) {
    run_case_status(body, expected, 0);
}

fn run_case_status(body: &str, expected: &str, status: i32) {
    for mode in ["interpreter", "jit", "aot", "aot-release"] {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("data"), "hello").unwrap();
        fs::write(dir.path().join("nul"), b"a\0b").unwrap();
        fs::create_dir(dir.path().join("folder")).unwrap();
        fs::write(dir.path().join("folder/item"), "item").unwrap();
        let source = dir.path().join("probe.lisp");
        let program = if mode == "interpreter" {
            body.to_string()
        } else {
            format!("(defun main () {body})")
        };
        fs::write(&source, program).unwrap();
        let mut command = if mode.starts_with("aot") {
            let binary = dir.path().join("probe");
            let mut compile = cargo_bin_cmd!("dlisp");
            compile
                .env("XDG_STATE_HOME", dir.path().join("state"))
                .arg("compile")
                .arg(&source)
                .arg("-o")
                .arg(&binary);
            if mode == "aot-release" {
                compile.arg("--release");
            }
            compile
                .timeout(Duration::from_secs(60))
                .assert()
                .success()
                .stderr("");
            Command::new(binary)
        } else {
            let mut cmd = cargo_bin_cmd!("dlisp");
            cmd.arg(&source);
            cmd
        };
        let output = command
            .current_dir(dir.path())
            .env("XDG_STATE_HOME", dir.path().join("state"))
            .timeout(Duration::from_secs(10))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(status),
            "{mode}: {output:?}\n{body}"
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{mode}: {body}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        if status == 0 {
            assert_eq!(stderr, "", "{mode}: {body}");
        } else {
            assert!(stderr.contains("Error"), "{mode}: {stderr}");
            assert!(!stderr.contains("panicked"), "{mode}: {stderr}");
        }
        if body.contains("(delete-file \"folder\")") {
            assert!(!dir.path().join("folder").exists(), "{mode}");
        }
    }
}

#[test]
fn argument_throws_stop_arithmetic_and_sh_without_running_later_arguments() {
    run_case(
        r#"(try (% (throw "first") (progn (print "never") 1))
                (catch e (print (error-value e))))
           (try (mod 1 (throw "second")) (catch e (print (error-value e))))
           (try (+ 1 (throw "last")) (catch e (print (error-value e))))
           (try (sh (throw "command") (progn (print "never") "arg"))
                (catch e (print (error-value e))))"#,
        "first\nsecond\nlast\ncommand\n",
    );
}

#[test]
fn uncaught_runtime_errors_exit_one_without_panicking() {
    run_case_status("(print (/ 1 0))", "", 1);
    run_case_status("(print (read-file 1))", "", 1);
}

#[test]
fn failed_io_is_not_replayed_by_interpreter_fallback() {
    run_case(
        r#"(try (sh "printf once >> marker; printf '\000'")
                (catch e (print (error? e))))
           (print (read-file "marker"))"#,
        "true\nonce\n",
    );
}

#[test]
fn arithmetic_errors_escape_callbacks_and_nested_function_calls() {
    run_case(
        r#"(try (let ((f (lambda () (/ 1 0)))) (f)) (catch e (print (error-value e))))
           (try (map (lambda (x) (/ 1 x)) (list 0 1))
                (catch e (print (error-value e))))
           (try (try (/ 1 0)) (catch e (print (error-value e))))
           (print (+ 2 3))
           (try (throw "fresh") (catch e (print (error-value e))))"#,
        "Division by zero\nDivision by zero\nDivision by zero\n5\nfresh\n",
    );
}

#[test]
fn io_functions_work_inside_compiled_functions() {
    run_case(
        r#"(print (read-file "data"))
           (print (file-exists? "data")) (print (is-file? "data"))
           (print (is-dir? "folder"))
           (print (= (list-dir "folder") (list "item")))
           (print (read-file "missing")) (print (file-exists? "missing"))
           (print (is-file? "missing")) (print (is-dir? "missing"))
           (print (delete-file "folder")) (print (delete-file "folder"))"#,
        "hello\ntrue\ntrue\ntrue\ntrue\nnil\nfalse\nfalse\nfalse\ntrue\nfalse\n",
    );
}

#[test]
fn sh_returns_a_map_and_evaluates_arguments_left_to_right() {
    run_case(
        r#"(let ((r (sh (progn (print "command") "printf")
                         (progn (print "format") "%s")
                         (progn (print "value") "ok"))))
              (print (type-of r)) (print (get r :status))
              (print (get r :stdout)) (print (get r :stderr)))
           (let ((r (sh "printf shell; printf err >&2; exit 7")))
              (print (get r :status)) (print (get r :stdout))
              (print (get r :stderr)))
           (print (get (sh "dlisp-no-such-command-78123" "arg") :status))"#,
        "command\nformat\nvalue\nmap\n0\nok\n\n7\nshell\nerr\n-1\n",
    );
}

#[test]
fn io_errors_are_caught_and_do_not_poison_later_calls() {
    run_case(
        r#"(try (read-file "folder") (catch e (print (error? e))))
           (try (list-dir "data") (catch e (print (error? e))))
           (try (read-file 1) (catch e (print (error? e))))
           (try (file-exists? 1) (catch e (print (error? e))))
           (try (is-dir? 1) (catch e (print (error? e))))
           (try (is-file? 1) (catch e (print (error? e))))
           (try (list-dir 1) (catch e (print (error? e))))
           (try (delete-file 1) (catch e (print (error? e))))
           (try (sh 1) (catch e (print (error? e))))
           (print (read-file "data"))
           (try (throw "later") (catch e (print (error-value e))))"#,
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nhello\nlater\n",
    );
}

#[test]
fn nul_io_results_are_catchable_errors() {
    run_case(
        r#"(try (read-file "nul") (catch e (print (error? e))))
           (try (sh "printf 'a\000b'") (catch e (print (error? e))))
           (try (sh "printf 'a\000b' >&2") (catch e (print (error? e))))"#,
        "true\ntrue\ntrue\n",
    );
}

#[test]
fn arithmetic_errors_are_caught_on_all_paths() {
    for expression in [
        "(+ 1 \"x\")",
        "(- \"x\")",
        "(* 1 \"x\")",
        "(/ 1 \"x\")",
        "(% 1 \"x\")",
        "(mod 1 \"x\")",
        "(+ \"x\")",
        "(* \"x\")",
    ] {
        run_case(
            &format!("(try {expression} (catch e (print (error-value e))))"),
            "Arguments must be numbers\n",
        );
    }
    for expression in [
        "(/ 1 0)",
        "(/ 1 0.0)",
        "(/ 1.0 0)",
        "(/ 1.0 -0.0)",
        "(% 1 0)",
        "(% 1 0.0)",
        "(mod 1.0 0)",
    ] {
        run_case(
            &format!("(try {expression} (catch e (print (error-value e))))"),
            "Division by zero\n",
        );
    }
}

#[test]
fn integer_overflow_is_catchable_on_all_paths() {
    for expression in [
        "(+ 9223372036854775807 1)",
        "(- -9223372036854775808 1)",
        "(* 9223372036854775807 2)",
        "(- -9223372036854775808)",
        "(/ -9223372036854775808 -1)",
        "(% -9223372036854775808 -1)",
        "(mod -9223372036854775808 -1)",
    ] {
        run_case(
            &format!("(try {expression} (catch e (print (error-value e))))"),
            "Integer overflow\n",
        );
    }
}

#[test]
fn arithmetic_evaluates_arguments_once_before_reporting_errors() {
    run_case(
        r#"(try (+ (progn (print "first") 1) "x" (progn (print "last") 2))
                (catch e (print (error-value e))))
           (try (+ (progn (print "before") 1) (throw "stop") (progn (print "never") 2))
                (catch e (print (error-value e))))
           (print (+)) (print (*)) (print (+ 2)) (print (* 3)) (print (- 4))
           (print (/ 100 2 5)) (print (+ 1 2.5))"#,
        "first\nlast\nArguments must be numbers\nbefore\nstop\n0\n1\n2\n3\n-4\n10\n3.5\n",
    );
}

#[test]
fn addition_preserves_interpreter_signed_zero_semantics() {
    run_case("(print (+ -0.0)) (print (+ -0.0 -0.0))", "0\n0\n");
}
