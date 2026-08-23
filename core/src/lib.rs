#![allow(clippy::mutable_key_type)]
// Core modules for dlisp

pub mod ast;
pub mod builtins;
pub mod codegen;
pub mod compiler;
pub mod environment;
pub mod eval_failure;
pub mod forms;
pub mod interpreter;
pub mod jit;
pub mod jit_runner;
pub mod macros;
pub mod parser;
