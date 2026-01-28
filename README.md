# dlisp

A JIT-compiling Lisp interpreter written in Rust.

## Development Environment
- **OS**: Linux
- **Language**: Rust (Cargo)

## Technology Stack
The project leverages the following key libraries:

### Core Code Generation & JIT
- **[Cranelift](https://cranelift.dev)** (v0.128.1): Used for Just-In-Time compilation of Lisp code to native machine code.
  - `cranelift`: Core code generator.
  - `cranelift-jit`: JIT execution engine.
  - `cranelift-module`: Module management for JIT.
  - `cranelift-frontend`: IR construction.

### Parsing
- **[Chumsky](https://github.com/zesterer/chumsky)** (v0.9): A parser combinator library used for parsing Lisp syntax.

### CLI & Interaction
- **[Rustyline](https://github.com/kkawakam/rustyline)** (v14.0): Provides a readline implementation for the REPL with history support.
- **[Dirs](https://github.com/dirs-dev/dirs-rs)** (v5.0): Used to handle XDG-compliant path resolution for history and logs.

### Logging & Utilities
- **[Tracing](https://github.com/tokio-rs/tracing)**: Framework for instrumenting Rust programs to collect structured, event-based diagnostic information.
- **[Anyhow](https://github.com/dtolnay/anyhow)** & **[Thiserror](https://github.com/dtolnay/thiserror)**: Error handling.

## System Requirements

To build and run `dlisp`, you need the following dependencies installed on your system:

- **Rust**: Latest stable version (via rustup).
- **Boehm GC**: `libgc-dev` (Debian/Ubuntu) or `bdwgc` (others).

```bash
# Ubuntu/Debian
sudo apt-get install libgc-dev pkg-config
```

## Usage

### Build
```bash
cargo build --workspace
```

### Run REPL
Start the interactive Read-Eval-Print Loop:
```bash
cargo run --bin cli
```

### Run Script
Execute a Lisp script file:
```bash
cargo run --bin cli -- path/to/script.lisp
```

### Compile to Native Executable
Compile a Lisp script into a standalone native executable:
```bash
cargo run --bin cli compile path/to/script.lisp -o my_app
./my_app
```

## Command-line Options

```text
dlisp [FILE] [COMMAND]

Arguments:
  [FILE]  Optional script file to execute if no subcommand is given

Commands:
  compile  Compile a script to a native executable
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Compile options:
  -o, --output <OUTPUT>  Output filename
```

## Configuration & Data

`dlisp` follows the XDG Base Directory Specification on Linux:

- **History**: REPL command history is saved in `~/.local/state/dlisp/history.txt`.
- **Logs**: Debug logs are stored in `~/.local/state/dlisp/debug.log`.

You can control logging levels via the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run --bin cli
```

## Quick Examples

### Arithmetic & Functions
```lisp
user> (+ 1 2 (* 3 4))
=> 15
user> (defun square (x) (* x x))
=> <user-func:square>
user> (square 5)
=> 25
```

### Concurrent Tasks
```lisp
user> (defun async-task () (print "Hello from thread!"))
=> <user-func:async-task>
user> (spawn async-task)
=> nil
```

## Language Features

### Data Types
- **Integers**: `1`, `42`, `-10`
- **Floats**: `3.14`, `-0.5`
- **Symbols**: `x`, `foo-bar`, `+`
- **Lists**: `(1 2 3)`, `(print "hello")`

### Special Forms
- **`defun`**: Define global functions.
- **`let`**: Bind local variables.
- **`lambda`**: Create anonymous functions (closures).
- **`if`**: Conditional execution.
- **`spawn`**: Spawn a new concurrent thread for a task.

### Built-in Functions
- **Arithmetic**: `+`, `-`, `*`
- **Comparison**: `>`
- **I/O**: `print`
- **System**: `sleep`

## Architecture Highlights

- **Garbage Collection**: Uses **Boehm GC** (`libgc`) for automatic memory management, ensuring thread safety and preventing leaks in both interpreted and compiled code.
- **JIT & AOT**: Shares a common Cranelift backend for both Just-In-Time execution in the REPL and Ahead-of-Time compilation to native binaries.
- **Concurrency**: Native OS threads spawned via `spawn`, heavily stress-tested for GC safety across thread boundaries.

## Features
- Interactive REPL with history support.
- JIT compilation for immediate feedback.
- AOT compilation to standalone executables.
- Asynchronous threaded execution.
- XDG-compliant configuration.
