# AI Agent Guide for dlisp

## Project Overview
`dlisp` is a JIT-compiling Lisp interpreter written in Rust. It aims to provide a performant Lisp environment by compiling user-defined functions to native machine code using Cranelift.

## Project Structure
The project is organized as a Cargo workspace with two members:

- **`cli`**: The command-line interface crate. Handles REPL, argument parsing, and file execution.
- **`core`**: The library crate containing the core interpreter logic.

### Core Modules (`core/src/`)
- **`ast.rs`**: Defines the `Value` enum, representing Lisp data types (Integer, Float, Bool, Symbol, String, List, Nil, NativeFunc, UserFunc).
- **`interpreter.rs`**: implementation of the `Interpreter` struct. Contains the `eval` loop, function application logic, and special form handling (`defun`, `spawn`).
- **`jit.rs`**: Manages JIT compilation using generic `cranelift`. Compiles arithmetic operations in user functions to native code.
- **`environment.rs`**: Manages variable scopes and bindings.
- **`parser.rs`**: Implements the parser using the `chumsky` library.
- **`builtins/`**: Contains built-in native functions (e.g., arithmetic operations).

## Build and execution
- **Build**: `cargo build`
- **Run REPL**: `cargo run --bin cli`
- **Run Script**: `cargo run --bin cli -- <filename>`
- **Test**: `cargo test`

## Key Concepts
- **Values**: All Lisp values are represented by the `Value` enum. `UserFunc` stores both the AST body and an optional JIT-compiled code pointer.
- **Async Execution**: The interpreter is async-first, leveraging `tokio` and `async_recursion`. The `spawn` special form allows concurrent execution.
- **JIT Compilation**: When `defun` is called, the interpreter attempts to compile the function body using Cranelift. If successful, subsequent calls use the native code path for performance.
- **REPL**: Powered by `rustyline`, supporting history and standard readline keybindings.
