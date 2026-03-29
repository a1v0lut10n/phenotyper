// SPDX-License-Identifier: Apache-2.0
//! Phenotyper Core — compiler library for the Phenotyper structural artifact definition language.
//!
//! This crate implements the full compilation pipeline:
//! - **Lexer**: Tokenization of `.pht` and `.md` sources
//! - **Parser**: Rustemo-based parser producing a span-annotated AST
//! - **AST**: Surface syntax tree data types
//! - **Symbol table**: Name collection and resolution
//! - **IR**: Normalized intermediate representation
//! - **Semantic validation**: Type, render, and generation validation
//! - **Codegen**: Rust source code generation

pub mod ast;
pub mod codegen;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod symbol;
