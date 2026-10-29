use assert_cmd::{cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::{fs, time::Duration};
use tempfile::TempDir;

#[test]
fn reader_prefix_depth_is_rejected_without_aborting_cli_or_compiler() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("deep.lisp");
    for prefix in ["'", "`", ",", ",@", "'("] {
        fs::write(&source, format!("{}x", prefix.repeat(65_536))).unwrap();
        for compile in [false, true] {
            let mut cmd = cargo_bin_cmd!("dlisp");
            if compile {
                cmd.arg("compile");
            }
            cmd.arg(&source)
                .env("XDG_STATE_HOME", dir.path().join("state"))
                .timeout(Duration::from_secs(10))
                .assert()
                .code(1)
                .stderr(predicate::str::contains("expression nesting too deep"))
                .stderr(predicate::str::contains("panicked").not());
        }
    }
}

#[test]
fn backquote_within_parser_limit_stops_at_evaluation_depth_limit() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("backquote.lisp");
    fs::write(&source, format!("{}x", "`".repeat(4096))).unwrap();
    for compile in [false, true] {
        let mut cmd = cargo_bin_cmd!("dlisp");
        if compile {
            cmd.arg("compile");
        }
        cmd.arg(&source)
            .env("XDG_STATE_HOME", dir.path().join("state"))
            .env("DLISP_MAX_EVAL_DEPTH", "256")
            .timeout(Duration::from_secs(10))
            .assert()
            .code(1)
            .stderr(predicate::str::contains("Maximum evaluation depth"))
            .stderr(predicate::str::contains("overflowed its stack").not());
    }
}

#[test]
fn decimal_and_unicode_reader_results_match_interpreter_jit_and_aot() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("reader.lisp");
    let binary = dir.path().join("reader");
    let body = r#"(print 0.001)
        (print (> 9223372036854775808.5 9000000000000000000.0))
        (print (> 0.12345678901234567890123456789 0.12))
        (let ((nilあ 7) (trueλ 8) (false猫 9)) (print (+ nilあ trueλ false猫)))"#;
    for mode in ["interpreter", "jit", "aot", "aot-release"] {
        fs::write(
            &source,
            if mode == "interpreter" {
                body.to_string()
            } else {
                format!("(defun main () {body})")
            },
        )
        .unwrap();
        let mut cmd = if mode.starts_with("aot") {
            let mut compile = cargo_bin_cmd!("dlisp");
            compile.arg("compile").arg(&source).arg("-o").arg(&binary);
            if mode == "aot-release" {
                compile.arg("--release");
            }
            compile
                .env("XDG_STATE_HOME", dir.path().join("state"))
                .timeout(Duration::from_secs(60))
                .assert()
                .success();
            Command::new(&binary)
        } else {
            let mut cmd = cargo_bin_cmd!("dlisp");
            cmd.arg(&source);
            cmd
        };
        cmd.env("XDG_STATE_HOME", dir.path().join("state"))
            .timeout(Duration::from_secs(10))
            .assert()
            .success()
            .stdout("0.001\ntrue\ntrue\n24\n")
            .stderr("");
    }
}

#[test]
fn numeric_helper_errors_are_catchable_and_aot_reports_unsupported_helpers() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("math.lisp");
    for body in [
        "(try (abs -9223372036854775808) (catch e (print (error-value e))))",
        "(print (pow 2 -4294967296))",
    ] {
        fs::write(&source, format!("(defun main () {body})")).unwrap();
        let expected = if body.contains("abs") {
            "Integer overflow\n"
        } else {
            "0\n"
        };
        cargo_bin_cmd!("dlisp")
            .arg(&source)
            .env("XDG_STATE_HOME", dir.path().join("state"))
            .timeout(Duration::from_secs(10))
            .assert()
            .success()
            .stdout(expected)
            .stderr("");
        cargo_bin_cmd!("dlisp")
            .arg("compile")
            .arg(&source)
            .env("XDG_STATE_HOME", dir.path().join("state"))
            .timeout(Duration::from_secs(10))
            .assert()
            .code(1)
            .stderr(predicate::str::contains(if body.contains("abs") {
                "abs"
            } else {
                "pow"
            }));
    }
}
