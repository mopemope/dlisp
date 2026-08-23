//! Failure channel for evaluation.
//!
//! `Message` is a regular error string (catchable by `try`).
//! `Recur` carries arguments back to the nearest enclosing `loop`; it must
//! not be intercepted by error handling while propagating.

use crate::ast::Value;
use std::fmt;

#[derive(Debug, Clone)]
pub enum EvalFailure {
    Message(String),
    Recur(Vec<Value>),
}

impl EvalFailure {
    pub fn message(msg: impl Into<String>) -> Self {
        EvalFailure::Message(msg.into())
    }
}

impl From<String> for EvalFailure {
    fn from(msg: String) -> Self {
        EvalFailure::Message(msg)
    }
}

impl fmt::Display for EvalFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalFailure::Message(msg) => write!(f, "{msg}"),
            // Reached the top level without an enclosing loop.
            EvalFailure::Recur(_) => write!(f, "recur outside loop"),
        }
    }
}

impl std::error::Error for EvalFailure {}
