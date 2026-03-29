// SPDX-License-Identifier: Apache-2.0
//! Diagnostic module — structured error, warning, and info messages.

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A compiler diagnostic with source location and human-readable messages.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub summary: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub explanation: Option<String>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.file, self.line, self.col, severity, self.summary
        )
    }
}
