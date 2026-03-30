// SPDX-License-Identifier: Apache-2.0
//! Diagnostic module — structured error, warning, and info messages.
//!
//! Provides the [`Diagnostic`] type used throughout the compiler pipeline,
//! along with human-readable and JSON formatters (REQ-COMP-010).
//!
//! # Formatting
//!
//! - **Human-readable** ([`format_human`]): GCC-style `file:line:col: severity: message`
//!   with indented explanation and suggestion lines.
//! - **JSON** ([`format_json`]): Machine-readable NDJSON for IDE integration and `--json` flag.
//! - **Summary** ([`format_summary`]): Compact one-line counts of errors/warnings.

#[cfg(test)]
mod tests;

// ─── Severity ───────────────────────────────────────────────────────────────

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational note.
    Info,
    /// Non-fatal warning.
    Warning,
    /// Fatal error — compilation will fail.
    Error,
}

impl Severity {
    /// Returns the lowercase label for this severity.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ─── Diagnostic ─────────────────────────────────────────────────────────────

/// A compiler diagnostic with source location and human-readable messages.
///
/// Diagnostics are produced by every phase of the compiler:
/// parser, symbol resolution, IR lowering, and semantic validation.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// How severe this issue is.
    pub severity: Severity,
    /// Concise one-line summary.
    pub summary: String,
    /// Source file path.
    pub file: String,
    /// 1-indexed line number (0 = unknown).
    pub line: usize,
    /// 1-indexed column number (0 = unknown).
    pub col: usize,
    /// Expanded explanation of the issue.
    pub explanation: Option<String>,
    /// Suggested fix for the issue.
    pub suggestion: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(file: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            summary: summary.into(),
            file: file.into(),
            line: 0,
            col: 0,
            explanation: None,
            suggestion: None,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(file: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            summary: summary.into(),
            file: file.into(),
            line: 0,
            col: 0,
            explanation: None,
            suggestion: None,
        }
    }

    /// Create a new info diagnostic.
    pub fn info(file: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            summary: summary.into(),
            file: file.into(),
            line: 0,
            col: 0,
            explanation: None,
            suggestion: None,
        }
    }

    /// Set the source location.
    pub fn at(mut self, line: usize, col: usize) -> Self {
        self.line = line;
        self.col = col;
        self
    }

    /// Set the explanation.
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }

    /// Set the fix suggestion.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Returns `true` if this is an error-level diagnostic.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// The `Display` impl produces the compact GCC-style one-liner.
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.file, self.line, self.col, self.severity, self.summary
        )
    }
}

// ─── Human-readable formatter ───────────────────────────────────────────────

/// Format a diagnostic in a rich human-readable style for terminal output.
///
/// Produces output like:
/// ```text
/// error: duplicate type name `Record`
///   --> test.pht:5:1
///   = note: a type with this name was already declared
///   = help: use a different name
/// ```
pub fn format_human(diag: &Diagnostic) -> String {
    let mut out = String::new();

    // Header: severity: summary
    out.push_str(diag.severity.label());
    out.push_str(": ");
    out.push_str(&diag.summary);
    out.push('\n');

    // Location
    if diag.line > 0 {
        out.push_str(&format!("  --> {}:{}:{}\n", diag.file, diag.line, diag.col));
    } else {
        out.push_str(&format!("  --> {}\n", diag.file));
    }

    // Explanation
    if let Some(ref explanation) = diag.explanation {
        out.push_str(&format!("  = note: {explanation}\n"));
    }

    // Suggestion
    if let Some(ref suggestion) = diag.suggestion {
        out.push_str(&format!("  = help: {suggestion}\n"));
    }

    out
}

/// Format a list of diagnostics for human-readable output, with a summary line.
pub fn format_human_all(diags: &[Diagnostic]) -> String {
    let mut out = String::new();
    for diag in diags {
        out.push_str(&format_human(diag));
        out.push('\n');
    }
    out.push_str(&format_summary(diags));
    out
}

// ─── JSON formatter ─────────────────────────────────────────────────────────

/// Format a diagnostic as a JSON object (single line, NDJSON-compatible).
///
/// Uses only the standard library — no serde dependency needed.
pub fn format_json(diag: &Diagnostic) -> String {
    let mut out = String::from("{");

    out.push_str(&format!("\"severity\":\"{}\",", diag.severity.label()));
    out.push_str(&format!("\"summary\":{},", json_escape(&diag.summary)));
    out.push_str(&format!("\"file\":{},", json_escape(&diag.file)));
    out.push_str(&format!("\"line\":{},", diag.line));
    out.push_str(&format!("\"col\":{}", diag.col));

    if let Some(ref explanation) = diag.explanation {
        out.push_str(&format!(",\"explanation\":{}", json_escape(explanation)));
    }
    if let Some(ref suggestion) = diag.suggestion {
        out.push_str(&format!(",\"suggestion\":{}", json_escape(suggestion)));
    }

    out.push('}');
    out
}

/// Format a list of diagnostics as NDJSON (one JSON object per line).
pub fn format_json_all(diags: &[Diagnostic]) -> String {
    let mut out = String::new();
    for diag in diags {
        out.push_str(&format_json(diag));
        out.push('\n');
    }
    out
}

// ─── Summary ────────────────────────────────────────────────────────────────

/// Format a compact summary line: `N error(s), M warning(s)`.
pub fn format_summary(diags: &[Diagnostic]) -> String {
    let errors = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    match (errors, warnings) {
        (0, 0) => "no errors".to_string(),
        (e, 0) => format!("{e} error{}", plural(e)),
        (0, w) => format!("{w} warning{}", plural(w)),
        (e, w) => format!("{e} error{}, {w} warning{}", plural(e), plural(w)),
    }
}

/// Count the number of errors in a diagnostic list.
pub fn error_count(diags: &[Diagnostic]) -> usize {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count()
}

/// Returns true if any diagnostic is an error.
pub fn has_errors(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| d.severity == Severity::Error)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Minimal JSON string escaping (no serde dependency).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
