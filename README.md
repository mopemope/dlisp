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

## Usage

### Build
```bash
cargo build
```

### Run REPL
```bash
cargo run --bin cli
```

### Run Tests
```bash
cargo test
```

## Features
- Interactive REPL with history (stored in `~/.local/state/dlisp/history.txt`).
- JIT compilation of arithmetic expressions.
- Debug logging (logs stored in `~/.local/state/dlisp/debug.log`).
