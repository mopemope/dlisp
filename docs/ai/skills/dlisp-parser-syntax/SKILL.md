---
name: dlisp-parser-syntax
description: Use when changing dlisp parser, AST, reader syntax, comments, literals, quote/backquote parsing, vectors, maps, keywords, or parse errors.
---

# dlisp Parser Syntax

- Start with `core/src/parser.rs` and `core/src/ast.rs`.
- Read [references/workflow.md](references/workflow.md) before changing syntax or parser tests.
- Prefer parser tests first; only inspect evaluator/codegen after confirming parsed `Value` shape.
- If syntax changes user-facing docs, also use `dlisp-language-surface`.
