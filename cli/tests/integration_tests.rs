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

#[test]
fn test_compile_supports_vector_literals_and_collection_parity() {
    // Regression: AOT used to panic while lowering `(vector ...)`, and
    // map/filter/reduce/nth/= diverged from the interpreter for vectors/lists.
    let (_dir, script, output) = write_temp_script(
        "compiled_vector_parity",
        r#"
(defun map-inc (xs)
  (map (lambda (x) (+ x 1)) xs))

(defun sum (xs)
  (reduce (lambda (a b) (+ a b)) 0 xs))

(defun main ()
  (print (vector 1 2 3))
  (print (map-inc (vector 1 2 3)))
  (print (sum (vector 1 2 3)))
  (print (= (list 1 2) (list 1 2)))
  (print (nth (list 7 8 9) 1)))
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
        .stdout(predicate::str::contains("[1 2 3]"))
        .stdout(predicate::str::contains("[2 3 4]"))
        .stdout(predicate::str::contains("6"))
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("8"));
}

#[test]
fn test_compile_supports_loops_and_higher_order_predicates() {
    // while/dotimes/dolist and some/every/find/for-each must compile to
    // native code with interpreter-identical results.
    let (_dir, script, output) = write_temp_script(
        "compiled_loops_parity",
        r#"
(defun w-sum (n)
  (let ((acc 0) (i 0))
    (while (< i n)
      (setq acc (+ acc i))
      (setq i (+ i 1)))
    acc))

(defun d-accum (n)
  (let ((acc 0))
    (dotimes (k n)
      (setq acc (+ acc k)))
    acc))

(defun dl-sum (xs)
  (let ((acc 0))
    (dolist (x xs)
      (setq acc (+ acc x)))
    acc))

(defun main ()
  (print (w-sum 5))
  (print (d-accum 5))
  (print (dl-sum '(1 2 3)))
  (print (dl-sum [4 5]))
  (print (some (lambda (x) (- x 2)) '(5 6)))
  (print (every (lambda (x) (> x 0)) '(1 2 3)))
  (print (find (lambda (x) (> x 2)) '(1 2 3 4)))
  (print (for-each (lambda (x) x) '(1 2 3))))
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
        .stdout(predicate::str::contains("6"))
        .stdout(predicate::str::contains("9"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("nil"));
}

#[test]
fn test_script_supports_require_core() {
    let (_dir, script, _output) = write_temp_script(
        "require_core_script",
        r#"
(require "core")
(print (inc 4))
(print (when-let (x 6) (inc x)))
"#,
    );

    let mut cmd = dlisp_cmd();
    cmd.arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("7"));
}

#[test]
fn test_script_supports_relative_file_require() {
    let (dir, script, _output) = write_temp_script(
        "require_file_script",
        r#"
(require "./lib/math.lisp")

(defun main ()
  (print (triple 4)))
"#,
    );
    let lib_dir = dir.path().join("lib");
    fs::create_dir_all(&lib_dir).expect("failed to create lib dir");
    fs::write(
        lib_dir.join("math.lisp"),
        r#"
(defun triple (x) (* x 3))
"#,
    )
    .expect("failed to write required file");

    let mut cmd = dlisp_cmd();
    cmd.arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("12"));
}

#[test]
fn test_compile_supports_require_core() {
    let (_dir, script, output) = write_temp_script(
        "require_core_compiled",
        r#"
(require "core")

(defun main ()
  (print (inc 4))
  (print (when-let (x 6) (inc x))))
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
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("7"));
}

#[test]
fn test_compile_supports_relative_file_require_with_macro() {
    let (dir, script, output) = write_temp_script(
        "require_file_compiled",
        r#"
(require "./lib/macros.lisp")

(defun main ()
  (print (twice (plus-one 6))))
"#,
    );
    let lib_dir = dir.path().join("lib");
    fs::create_dir_all(&lib_dir).expect("failed to create lib dir");
    fs::write(
        lib_dir.join("macros.lisp"),
        r#"
(defun macro-add-one (x)
  (list '+ x 1))

(defmacro plus-one (x)
  (macro-add-one x))

(defun twice (x)
  (* x 2))
"#,
    )
    .expect("failed to write required file");

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
        .stdout(predicate::str::contains("14"));
}

#[test]
fn test_compile_supports_phase4_control_surface() {
    let (_dir, script, output) = write_temp_script(
        "compiled_phase4_surface",
        r#"
(require "core")

(defun main ()
  (print
    (progn
      (when true
        (print (list 1 2)))
      (unless false
        (print (first (cons 9 (list 8)))))
      (cond
        ((and (positive? 3) (not nil))
          (rest (list 1 2 3)))
        (true
          (list 0)))))
  (print (cons 11 22))
  (print (when-let (x 4) (inc x))))
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
        .stdout(predicate::str::contains("(1 2)"))
        .stdout(predicate::str::contains("9"))
        .stdout(predicate::str::contains("(2 3)"))
        .stdout(predicate::str::contains("(11 22)"))
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_compile_try_throw() {
    let (_dir, script, output) = write_temp_script(
        "compiled_try_throw",
        r#"
(defun thrower (v) (throw v))

(defun main ()
  (print (try (thrower "deep") (catch e (string-append "caught: " (error-value e)))))
  (print (try (+ 1 1) (catch e :never)))
  (print (try (thrower 99) (catch e (+ (error-value e) 1))))
  (print (type-of (try (thrower :kw) (catch e e))))
  (print (error? (try (thrower "x") (catch e e)))))
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
        .stdout(predicate::str::contains("caught: deep"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("100"))
        .stdout(predicate::str::contains("error"))
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_compile_try_throw_across_functions_and_loop() {
    let (_dir, script, output) = write_temp_script(
        "compiled_try_throw_boundary",
        r#"
(defun throw-if-negative (n) (if (< n 0) (throw :negative) (* n n)))
(defun safe-sq (n) (try (throw-if-negative n) (catch e (error-value e))))

(defun main ()
  (print (safe-sq 5))
  (print (safe-sq -3))
  (let ((g (lambda () (throw :from-lambda))))
    (print (try (g) (catch e (error-value e)))))
  (let ((i 0) (caught 0))
    (while (< i 10)
      (try (if (= (% i 2) 0) (throw :even) i) (catch e (setq caught (+ caught 1))))
      (setq i (+ i 1)))
    (print caught)))
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
        .stdout(predicate::str::contains("25"))
        .stdout(predicate::str::contains(":negative"))
        .stdout(predicate::str::contains(":from-lambda"))
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_compile_uncaught_throw_exits_nonzero() {
    // An uncaught throw escaping main must fail the compiled program the
    // same way the interpreter fails the script (nonzero exit), not exit 0
    // silently.
    let (_dir, script, output) = write_temp_script(
        "compiled_uncaught_throw",
        r#"
(defun main ()
  (print :before)
  (throw :boom)
  (print :after))
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
        .failure()
        .stdout(predicate::str::contains(":before"))
        .stdout(predicate::str::contains(":after").not());
}
