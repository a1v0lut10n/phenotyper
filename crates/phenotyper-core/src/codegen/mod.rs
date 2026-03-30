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
pub fn generate(module: &PhenotypeModule) -> String {
    emit::emit_module(module)
}
