// SPDX-License-Identifier: Apache-2.0
//! Phenotyper CLI — compiler binary for the Phenotyper language.
//!
//! Subcommands:
//! - `check` — validate a source file without generating code (REQ-CLI-002)
//! - `build` — validate and generate Rust code (REQ-CLI-002)
//! - `dump-ast` — print the parsed AST for debugging (REQ-CLI-003)
//! - `dump-ir` — print the normalized IR for debugging (REQ-CLI-003)

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use phenotyper::diagnostic::{self, Diagnostic};

#[derive(Parser)]
#[command(name = "phenotyper", about = "Phenotyper v1 compiler", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a phenotyper source file for errors without generating code.
    Check {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output diagnostics as JSON
        #[arg(long)]
        json: bool,
    },

    /// Compile a phenotyper source file and generate Rust code.
    Build {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output directory for generated Rust code
        #[arg(long, short, value_name = "DIR")]
        out: PathBuf,

        /// Output diagnostics as JSON
        #[arg(long)]
        json: bool,
    },

    /// Dump the AST of a source file (debug).
    DumpAst {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Dump the IR of a source file (debug).
    DumpIr {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

// ─── Exit codes (REQ-CLI-005) ───────────────────────────────────────────────

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1;
// const EXIT_USAGE: u8 = 2; // Handled by clap automatically

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { file, json } => cmd_check(&file, json),
        Commands::Build { file, out, json } => cmd_build(&file, &out, json),
        Commands::DumpAst { file } => cmd_dump_ast(&file),
        Commands::DumpIr { file } => cmd_dump_ir(&file),
    }
}

// ─── check ──────────────────────────────────────────────────────────────────

fn cmd_check(file: &Path, json: bool) -> ExitCode {
    let file_str = file.display().to_string();

    let source = match read_source(file) {
        Ok(s) => s,
        Err(e) => {
            emit_single_error(&file_str, &e, json);
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    // Use compile_source with no output directory (check only)
    match phenotyper::compile_source(&source, &file_str, None) {
        Ok(output) => {
            emit_diagnostics(&output.warnings, json);
            if !json {
                eprintln!(
                    "✓ {} — {}",
                    file_str,
                    diagnostic::format_summary(&output.warnings)
                );
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(errors) => {
            emit_diagnostics(&errors, json);
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

// ─── build ──────────────────────────────────────────────────────────────────

fn cmd_build(file: &Path, out: &Path, json: bool) -> ExitCode {
    let file_str = file.display().to_string();

    let source = match read_source(file) {
        Ok(s) => s,
        Err(e) => {
            emit_single_error(&file_str, &e, json);
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    // Use compile_source with output directory
    match phenotyper::compile_source(&source, &file_str, Some(out)) {
        Ok(output) => {
            emit_diagnostics(&output.warnings, json);
            if !json {
                eprintln!(
                    "✓ {} → {} — {}",
                    file_str,
                    out.display(),
                    diagnostic::format_summary(&output.warnings)
                );
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(errors) => {
            emit_diagnostics(&errors, json);
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

// ─── dump-ast ───────────────────────────────────────────────────────────────

fn cmd_dump_ast(file: &Path) -> ExitCode {
    let file_str = file.display().to_string();

    let source = match read_source(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    match phenotyper::parser::parse_pht(&source, &file_str) {
        Ok(ast) => {
            println!("{ast:#?}");
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", diagnostic::format_human(d));
            }
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

// ─── dump-ir ────────────────────────────────────────────────────────────────

fn cmd_dump_ir(file: &Path) -> ExitCode {
    let file_str = file.display().to_string();

    let source = match read_source(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    // Parse
    let ast = match phenotyper::parser::parse_pht(&source, &file_str) {
        Ok(ast) => ast,
        Err(diags) => {
            for d in &diags {
                eprintln!("{}", diagnostic::format_human(d));
            }
            return ExitCode::from(EXIT_FAILURE);
        }
    };

    // Symbol resolution
    let (table, sym_diags) = phenotyper::symbol::build(&ast, &file_str);
    if diagnostic::has_errors(&sym_diags) {
        for d in &sym_diags {
            eprintln!("{}", diagnostic::format_human(d));
        }
        return ExitCode::from(EXIT_FAILURE);
    }

    // IR lowering
    let (module, ir_diags) = phenotyper::ir::lower(&ast, &table, &file_str);
    if diagnostic::has_errors(&ir_diags) {
        for d in &ir_diags {
            eprintln!("{}", diagnostic::format_human(d));
        }
        return ExitCode::from(EXIT_FAILURE);
    }

    println!("{module:#?}");
    ExitCode::from(EXIT_SUCCESS)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Read source from a file, handling .pht and .md extensions (REQ-CLI-006).
fn read_source(file: &Path) -> Result<String, String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "pht" | "md" => {}
        _ => {
            return Err(format!(
                "unsupported file extension '.{ext}' — expected .pht or .md"
            ));
        }
    }

    std::fs::read_to_string(file).map_err(|e| format!("cannot read '{}': {e}", file.display()))
}

/// Emit diagnostics to the appropriate output (REQ-CLI-007).
fn emit_diagnostics(diags: &[Diagnostic], json: bool) {
    if diags.is_empty() {
        return;
    }

    if json {
        // JSON to stdout
        print!("{}", diagnostic::format_json_all(diags));
    } else {
        // Human-readable to stderr
        for d in diags {
            eprint!("{}", diagnostic::format_human(d));
        }
    }
}

/// Emit a single error diagnostic.
fn emit_single_error(file: &str, message: &str, json: bool) {
    let diag = Diagnostic::error(file, message);
    emit_diagnostics(&[diag], json);
}
