// SPDX-License-Identifier: Apache-2.0
//! Diagnostic formatting tests.

use super::*;

// ─── Severity ───────────────────────────────────────────────────────────────

#[test]
fn severity_label() {
    assert_eq!(Severity::Error.label(), "error");
    assert_eq!(Severity::Warning.label(), "warning");
    assert_eq!(Severity::Info.label(), "info");
}

#[test]
fn severity_display() {
    assert_eq!(format!("{}", Severity::Error), "error");
    assert_eq!(format!("{}", Severity::Warning), "warning");
    assert_eq!(format!("{}", Severity::Info), "info");
}

#[test]
fn severity_ordering() {
    assert!(Severity::Error > Severity::Warning);
    assert!(Severity::Warning > Severity::Info);
}

// ─── Diagnostic constructors ────────────────────────────────────────────────

#[test]
fn error_constructor() {
    let d = Diagnostic::error("test.pht", "something broke");
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.summary, "something broke");
    assert_eq!(d.file, "test.pht");
    assert_eq!(d.line, 0);
    assert_eq!(d.col, 0);
    assert!(d.is_error());
}

#[test]
fn warning_constructor() {
    let d = Diagnostic::warning("test.pht", "be careful");
    assert_eq!(d.severity, Severity::Warning);
    assert!(!d.is_error());
}

#[test]
fn builder_chain() {
    let d = Diagnostic::error("test.pht", "duplicate name")
        .at(10, 5)
        .with_explanation("a type with this name already exists")
        .with_suggestion("rename one of the types");
    assert_eq!(d.line, 10);
    assert_eq!(d.col, 5);
    assert_eq!(
        d.explanation.as_deref(),
        Some("a type with this name already exists")
    );
    assert_eq!(d.suggestion.as_deref(), Some("rename one of the types"));
}

// ─── Display (compact GCC-style) ────────────────────────────────────────────

#[test]
fn display_format() {
    let d = Diagnostic::error("test.pht", "parse error").at(5, 12);
    assert_eq!(format!("{d}"), "test.pht:5:12: error: parse error");
}

#[test]
fn display_zero_location() {
    let d = Diagnostic::warning("file.pht", "unused import");
    assert_eq!(format!("{d}"), "file.pht:0:0: warning: unused import");
}

// ─── Human-readable formatter ───────────────────────────────────────────────

#[test]
fn human_basic() {
    let d = Diagnostic::error("test.pht", "unknown type `Foo`").at(10, 5);
    let output = format_human(&d);
    assert!(output.contains("error: unknown type `Foo`"));
    assert!(output.contains("  --> test.pht:10:5"));
}

#[test]
fn human_with_explanation_and_suggestion() {
    let d = Diagnostic::error("test.pht", "duplicate field `name`")
        .at(15, 1)
        .with_explanation("a field with this name already exists in this type")
        .with_suggestion("use a different field name");
    let output = format_human(&d);
    assert!(output.contains("error: duplicate field `name`"));
    assert!(output.contains("  --> test.pht:15:1"));
    assert!(output.contains("  = note: a field with this name already exists"));
    assert!(output.contains("  = help: use a different field name"));
}

#[test]
fn human_no_location() {
    let d = Diagnostic::info("test.pht", "compilation complete");
    let output = format_human(&d);
    assert!(output.contains("info: compilation complete"));
    assert!(output.contains("  --> test.pht"));
    // Should NOT contain line:col
    assert!(!output.contains("test.pht:0:0"));
}

#[test]
fn human_all_with_summary() {
    let diags = vec![
        Diagnostic::error("a.pht", "err1"),
        Diagnostic::warning("a.pht", "warn1"),
        Diagnostic::error("a.pht", "err2"),
    ];
    let output = format_human_all(&diags);
    assert!(output.contains("error: err1"));
    assert!(output.contains("warning: warn1"));
    assert!(output.contains("error: err2"));
    assert!(output.contains("2 errors, 1 warning"));
}

// ─── JSON formatter ─────────────────────────────────────────────────────────

#[test]
fn json_basic() {
    let d = Diagnostic::error("test.pht", "parse error").at(5, 12);
    let json = format_json(&d);
    assert!(json.contains("\"severity\":\"error\""));
    assert!(json.contains("\"summary\":\"parse error\""));
    assert!(json.contains("\"file\":\"test.pht\""));
    assert!(json.contains("\"line\":5"));
    assert!(json.contains("\"col\":12"));
    // Should NOT contain explanation/suggestion when absent
    assert!(!json.contains("\"explanation\""));
    assert!(!json.contains("\"suggestion\""));
}

#[test]
fn json_with_explanation() {
    let d = Diagnostic::error("f.pht", "bad").with_explanation("it's really bad");
    let json = format_json(&d);
    assert!(json.contains("\"explanation\":\"it's really bad\""));
}

#[test]
fn json_escaping() {
    let d = Diagnostic::error("f.pht", "string with \"quotes\" and\nnewline");
    let json = format_json(&d);
    assert!(json.contains("\\\"quotes\\\""));
    assert!(json.contains("\\n"));
}

#[test]
fn json_ndjson_format() {
    let diags = vec![
        Diagnostic::error("a.pht", "err1"),
        Diagnostic::warning("a.pht", "warn1"),
    ];
    let output = format_json_all(&diags);
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with('{'));
    assert!(lines[0].ends_with('}'));
    assert!(lines[1].starts_with('{'));
}

// ─── Summary ────────────────────────────────────────────────────────────────

#[test]
fn summary_no_errors() {
    assert_eq!(format_summary(&[]), "no errors");
}

#[test]
fn summary_one_error() {
    let diags = vec![Diagnostic::error("f", "e")];
    assert_eq!(format_summary(&diags), "1 error");
}

#[test]
fn summary_multiple_errors() {
    let diags = vec![Diagnostic::error("f", "e1"), Diagnostic::error("f", "e2")];
    assert_eq!(format_summary(&diags), "2 errors");
}

#[test]
fn summary_warnings_only() {
    let diags = vec![Diagnostic::warning("f", "w")];
    assert_eq!(format_summary(&diags), "1 warning");
}

#[test]
fn summary_mixed() {
    let diags = vec![
        Diagnostic::error("f", "e"),
        Diagnostic::warning("f", "w1"),
        Diagnostic::warning("f", "w2"),
    ];
    assert_eq!(format_summary(&diags), "1 error, 2 warnings");
}

#[test]
fn summary_info_not_counted() {
    let diags = vec![Diagnostic::info("f", "i")];
    assert_eq!(format_summary(&diags), "no errors");
}

// ─── Utility functions ──────────────────────────────────────────────────────

#[test]
fn error_count_works() {
    let diags = vec![
        Diagnostic::error("f", "e1"),
        Diagnostic::warning("f", "w"),
        Diagnostic::error("f", "e2"),
    ];
    assert_eq!(error_count(&diags), 2);
}

#[test]
fn has_errors_works() {
    assert!(!has_errors(&[]));
    assert!(!has_errors(&[Diagnostic::warning("f", "w")]));
    assert!(has_errors(&[Diagnostic::error("f", "e")]));
}
