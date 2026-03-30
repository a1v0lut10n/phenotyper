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
//!
//! # Build script integration
//!
//! For `build.rs` integration (like `prost-build` or `rustemo`), use the
//! high-level [`compile()`] function:
//!
//! ```ignore
//! // In your build.rs:
//! fn main() {
//!     phenotyper_core::compile("src/phenotypes/csv.pht", "src/generated")
//!         .expect("phenotyper compilation failed");
//! }
//! ```

pub mod ast;
pub mod codegen;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod symbol;

use std::path::Path;

use diagnostic::Diagnostic;

// ─── High-level compile API ─────────────────────────────────────────────────

/// Result type for compilation.
pub type CompileResult = Result<CompileOutput, Vec<Diagnostic>>;

/// Successful compilation output.
#[derive(Debug)]
pub struct CompileOutput {
    /// The generated Rust source code.
    pub code: String,
    /// The namespace path (e.g., `"aivolution/format/csv"`).
    pub namespace: String,
    /// Any warnings generated during compilation.
    pub warnings: Vec<Diagnostic>,
}

/// Compile a Phenotyper source file and generate Rust code.
///
/// This is the primary entry point for `build.rs` integration. It runs the
/// full compilation pipeline (parse → symbols → IR → validate → codegen)
/// and returns the generated Rust code.
///
/// # Arguments
///
/// * `source_path` — Path to the `.pht` or `.md` source file.
/// * `out_dir` — Directory where generated Rust code will be written.
///
/// # Cargo integration
///
/// This function automatically prints `cargo:rerun-if-changed` directives
/// to stdout so Cargo will re-run the build script when the source changes.
///
/// # Errors
///
/// Returns a `Vec<Diagnostic>` if any phase produces errors.
///
/// # Example
///
/// ```ignore
/// // build.rs
/// fn main() {
///     phenotyper_core::compile("src/phenotypes/csv.pht", "src/generated")
///         .expect("phenotyper compilation failed");
/// }
/// ```
pub fn compile(source_path: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> CompileResult {
    let source_path = source_path.as_ref();
    let out_dir = out_dir.as_ref();
    let file_str = source_path.display().to_string();

    // Emit cargo:rerun-if-changed for build.rs integration
    println!("cargo:rerun-if-changed={file_str}");

    // Read source
    let source = std::fs::read_to_string(source_path)
        .map_err(|e| vec![Diagnostic::error(&file_str, format!("cannot read: {e}"))])?;

    compile_source(&source, &file_str, Some(out_dir))
}

/// Compile Phenotyper source code from a string (no file I/O for the source).
///
/// Useful for testing or embedding. If `out_dir` is `Some`, the generated
/// code is written to the filesystem; otherwise it is only returned.
pub fn compile_source(source: &str, file_name: &str, out_dir: Option<&Path>) -> CompileResult {
    let mut warnings = Vec::new();

    // Parse — auto-detect .md vs .pht by file extension
    let is_md = file_name.ends_with(".md");
    let ast = if is_md {
        parser::parse_md(source, file_name)?
    } else {
        parser::parse_pht(source, file_name)?
    };

    // Symbol resolution
    let (table, sym_diags) = symbol::build(&ast, file_name);
    let (sym_errors, sym_warnings): (Vec<_>, Vec<_>) =
        sym_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(sym_warnings);
    if !sym_errors.is_empty() {
        return Err(sym_errors);
    }

    // IR lowering
    let (module, ir_diags) = ir::lower(&ast, &table, file_name);
    let (ir_errors, ir_warnings): (Vec<_>, Vec<_>) =
        ir_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(ir_warnings);
    if !ir_errors.is_empty() {
        return Err(ir_errors);
    }

    // Semantic validation
    let sem_diags = semantic::validate(&module, file_name);
    let (sem_errors, sem_warnings): (Vec<_>, Vec<_>) =
        sem_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(sem_warnings);
    if !sem_errors.is_empty() {
        return Err(sem_errors);
    }

    // Code generation
    let code = codegen::generate(&module);

    // Write output if directory specified
    if let Some(out_dir) = out_dir {
        let ns_path = module.namespace.replace('/', std::path::MAIN_SEPARATOR_STR);
        let dir = out_dir.join(&ns_path);
        let file = dir.join("mod.rs");

        std::fs::create_dir_all(&dir).map_err(|e| {
            vec![Diagnostic::error(
                file_name,
                format!("cannot create directory '{}': {e}", dir.display()),
            )]
        })?;

        std::fs::write(&file, &code).map_err(|e| {
            vec![Diagnostic::error(
                file_name,
                format!("cannot write '{}': {e}", file.display()),
            )]
        })?;
    }

    Ok(CompileOutput {
        code,
        namespace: module.namespace.clone(),
        warnings,
    })
}
