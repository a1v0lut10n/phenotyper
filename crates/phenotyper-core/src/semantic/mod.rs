// SPDX-License-Identifier: Apache-2.0
//! Semantic validation module — type, render, and generation validation over the IR.
//!
//! Validates the lowered IR for semantic correctness before code generation:
//!
//! - **Type validation**: cardinality/render compatibility, plural integrity
//! - **Render validation**: directive constraints, optional field guards, collection guards
//! - **Generation validation**: cyclic type detection, union shape validity
//!
//! # Usage
//!
//! ```ignore
//! use phenotyper_core::semantic;
//! let diagnostics = semantic::validate(&module, "file.pht");
//! ```

mod validate;

#[cfg(test)]
mod tests;

use crate::diagnostic::{Diagnostic, Severity};

// ─── Entry point ────────────────────────────────────────────────────────────

/// Validate a lowered `PhenotypeModule` for semantic correctness.
///
/// Returns a list of diagnostics (errors and warnings).
/// An empty list means the module is valid and ready for code generation.
pub fn validate(module: &crate::ir::PhenotypeModule, file: &str) -> Vec<Diagnostic> {
    validate::validate_module(module, file)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn error(file: &str, summary: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: None,
        suggestion: None,
    }
}

fn error_with_explanation(file: &str, summary: String, explanation: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: Some(explanation),
        suggestion: None,
    }
}

fn error_with_suggestion(
    file: &str,
    summary: String,
    explanation: String,
    suggestion: String,
) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: Some(explanation),
        suggestion: Some(suggestion),
    }
}

#[allow(dead_code)]
fn warning(file: &str, summary: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: None,
        suggestion: None,
    }
}
