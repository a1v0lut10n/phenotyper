// SPDX-License-Identifier: Apache-2.0
//! Code generation module — Rust source code emission from the validated IR.
//!
//! Generates idiomatic Rust code from a `PhenotypeModule`:
//!
//! - **Singular structs** with public fields (REQ-CODEGEN-002)
//! - **Plural wrapper structs** with collection API (REQ-CODEGEN-003)
//! - **Builder structs** with `build()` validation (REQ-CODEGEN-004)
//! - **`Render` trait** implementations (REQ-CODEGEN-006)
//! - **Union enums** and **namespace enums** (REQ-CODEGEN-007/008)
//! - **`BuildError`** enum (REQ-CODEGEN-005)
//!
//! # Usage
//!
//! ```ignore
//! use phenotyper_core::codegen;
//! let rust_code = codegen::generate(&module);
//! ```

mod emit;
mod naming;

#[cfg(test)]
mod tests;

use crate::ir::PhenotypeModule;

// ─── Entry point ────────────────────────────────────────────────────────────

/// Generate Rust source code from a validated `PhenotypeModule`.
///
/// Returns a `String` containing the complete generated Rust module.
/// The output is post-processed with `rustfmt` if available (T-094).
pub fn generate(module: &PhenotypeModule) -> String {
    let raw = emit::emit_module(module);
    rustfmt(&raw).unwrap_or(raw)
}

/// Attempt to format Rust source code using `rustfmt`.
///
/// Returns `Some(formatted)` on success, or `None` if `rustfmt` is
/// unavailable or fails (graceful fallback).
fn rustfmt(source: &str) -> Option<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    child.stdin.as_mut()?.write_all(source.as_bytes()).ok()?;

    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
