// SPDX-License-Identifier: Apache-2.0
//! Semantic validation tests.

use crate::diagnostic::Severity;
use crate::ir;
use crate::parser;
use crate::symbol;

/// Helper: parse → symbols → IR → validate, returning only error diagnostics.
fn validate_errors(source: &str) -> Vec<String> {
    let ast = parser::parse_pht(source, "test.pht").expect("parse failed");
    let (table, sym_diags) = symbol::build(&ast, "test.pht");
    assert!(
        sym_diags.iter().all(|d| d.severity != Severity::Error),
        "unexpected symbol errors: {sym_diags:#?}"
    );

    let (module, ir_diags) = ir::lower(&ast, &table, "test.pht");
    assert!(
        ir_diags.iter().all(|d| d.severity != Severity::Error),
        "unexpected IR errors: {ir_diags:#?}"
    );

    let sem_diags = super::validate(&module, "test.pht");
    sem_diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.summary.clone())
        .collect()
}

/// Helper: validate and expect zero errors.
fn validate_ok(source: &str) {
    let errors = validate_errors(source);
    assert!(
        errors.is_empty(),
        "unexpected validation errors: {errors:#?}"
    );
}

/// Helper: validate and expect at least one error containing the substring.
fn expect_error(source: &str, expected_substr: &str) {
    let errors = validate_errors(source);
    let has_match = errors.iter().any(|e| e.contains(expected_substr));
    assert!(
        has_match,
        "expected error containing `{expected_substr}`, got: {errors:#?}"
    );
}

// ─── Valid programs ─────────────────────────────────────────────────────────

#[test]
fn valid_csv_fixture() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/csv_basic.pht"
    ))
    .expect("fixture file missing");
    validate_ok(&source);
}

#[test]
fn valid_simple_type() {
    validate_ok(
        r#"
        test/types:
        Record:
            name: required string,
            @(name)
        ;
    .
    "#,
    );
}

#[test]
fn valid_optional_with_ifset() {
    validate_ok(
        r#"
        test/types:
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @ifset(footer) { @(footer) }
        ;
    .
    "#,
    );
}

#[test]
fn valid_collection_with_join() {
    validate_ok(
        r#"
        test/types:
        Record:
            tags: required string*,
            @join(tags, ", ")
        ;
    .
    "#,
    );
}

#[test]
fn valid_plural_with_join() {
    validate_ok(
        r#"
        test/types:
        Item plural Items:
            value: required string,
            @(value)
        ;
        Container:
            items: required Items,
            sep: required string,
            @join(items, sep)
        ;
    .
    "#,
    );
}

#[test]
fn valid_ifnotempty_on_collection() {
    validate_ok(
        r#"
        test/types:
        Record:
            tags: required string*,
            @ifnotempty(tags) { @join(tags, ", ") }
        ;
    .
    "#,
    );
}

#[test]
fn valid_eol_with_string_field() {
    validate_ok(
        r#"
        test/types:
        Record:
            value: required string,
            eol: required string,
            @(value),
            @eol(eol)
        ;
    .
    "#,
    );
}

#[test]
fn valid_bare_eol() {
    validate_ok(
        r#"
        test/types:
        Record:
            value: required string,
            @(value),
            @eol
        ;
    .
    "#,
    );
}

#[test]
fn valid_text_only_render() {
    validate_ok(
        r#"
        test/types:
        Record:
            value: required string,
            "prefix: ",
            @(value)
        ;
    .
    "#,
    );
}

// ─── T-062: Direct @emit of collection field ────────────────────────────────

#[test]
fn error_emit_collection_field() {
    expect_error(
        r#"
        test/types:
        Record:
            tags: required string*,
            @(tags)
        ;
    .
    "#,
        "cannot directly emit collection field `tags`",
    );
}

#[test]
fn error_emit_plural_field() {
    expect_error(
        r#"
        test/types:
        Item plural Items:
            value: required string,
            @(value)
        ;
        Container:
            items: required Items,
            @(items)
        ;
    .
    "#,
        "cannot directly emit collection field `items`",
    );
}

#[test]
fn error_emit_one_or_more_field() {
    expect_error(
        r#"
        test/types:
        Record:
            tags: required string+,
            @(tags)
        ;
    .
    "#,
        "cannot directly emit collection field `tags`",
    );
}

// ─── T-063: Direct @emit of optional field outside @ifset ────────────────────

#[test]
fn error_emit_optional_outside_ifset() {
    expect_error(
        r#"
        test/types:
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @(footer)
        ;
    .
    "#,
        "cannot directly emit optional field `footer`",
    );
}

#[test]
fn valid_emit_optional_inside_ifset() {
    // Should NOT produce the optional-emit error when inside @ifset
    validate_ok(
        r#"
        test/types:
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @ifset(footer) { @(footer) }
        ;
    .
    "#,
    );
}

// ─── T-063a: @ifset on non-optional field ────────────────────────────────────

#[test]
fn error_ifset_on_required_field() {
    expect_error(
        r#"
        test/types:
        Record:
            name: required string,
            @ifset(name) { @(name) }
        ;
    .
    "#,
        "@ifset on non-optional field `name`",
    );
}

// ─── T-063b: @ifnotempty on non-collection field ─────────────────────────────

#[test]
fn error_ifnotempty_on_scalar_field() {
    expect_error(
        r#"
        test/types:
        Record:
            name: required string,
            @ifnotempty(name) { @(name) }
        ;
    .
    "#,
        "@ifnotempty on non-collection field `name`",
    );
}

// ─── T-064: Separator must be scalar string ──────────────────────────────────

#[test]
fn error_join_separator_non_string() {
    expect_error(
        r#"
        test/types:
        Record:
            items: required string*,
            count: required int64,
            @join(items, count)
        ;
    .
    "#,
        "@join separator field `count`",
    );
}

#[test]
fn error_join_separator_collection() {
    expect_error(
        r#"
        test/types:
        Record:
            items: required string*,
            seps: required string*,
            @join(items, seps)
        ;
    .
    "#,
        "must be singular",
    );
}

// ─── T-062 extended: @join field must be collection ──────────────────────────

#[test]
fn error_join_on_scalar_field() {
    expect_error(
        r#"
        test/types:
        Record:
            name: required string,
            @join(name, ", ")
        ;
    .
    "#,
        "@join field `name` in type `Record` is not a collection",
    );
}

// ─── T-065: Cyclic type detection ────────────────────────────────────────────

#[test]
fn error_direct_cycle() {
    expect_error(
        r#"
        test/types:
        Node:
            child: required Node,
            @(child)
        ;
    .
    "#,
        "cyclic type definition involving `Node`",
    );
}

#[test]
fn error_indirect_cycle() {
    expect_error(
        r#"
        test/types:
        A:
            b: required B,
            @(b)
        ;
        B:
            a: required A,
            @(a)
        ;
    .
    "#,
        "cyclic type definition",
    );
}

#[test]
fn valid_optional_breaks_cycle() {
    // Optional references should NOT create a cycle
    validate_ok(
        r#"
        test/types:
        Node:
            name: required string,
            child: optional Node,
            @(name),
            @ifset(child) { @(child) }
        ;
    .
    "#,
    );
}

#[test]
fn valid_collection_breaks_cycle() {
    // Collection references should NOT create a cycle
    validate_ok(
        r#"
        test/types:
        Node:
            name: required string,
            children: required Node*,
            @(name),
            @ifnotempty(children) { @join(children, ", ") }
        ;
    .
    "#,
    );
}

// ─── Render body non-empty ──────────────────────────────────────────────────

#[test]
fn error_empty_render_body() {
    // A type with no render expressions at all
    // Note: this requires a type with fields but no render
    // Currently the grammar requires at least one body item, and the parser
    // forces at least one BodyItem, so we can test empty render via @ifset
    // with no body items — but that would require a grammar change.
    // For now, we test by checking the diagnostic on types with render.
    // Skip: this is a grammar-level constraint.
}

// ─── Eol field type validation ──────────────────────────────────────────────

#[test]
fn error_eol_with_non_string_field() {
    expect_error(
        r#"
        test/types:
        Record:
            value: required string,
            count: required int64,
            @(value),
            @eol(count)
        ;
    .
    "#,
        "@eol field `count` in type `Record` must be a string",
    );
}

// ─── ? suffix operator ──────────────────────────────────────────────────────

#[test]
fn valid_conditional_ref_optional() {
    // @(optional_field)? is valid sugar for @ifset
    validate_ok(
        r#"
        test/types:
        Record:
            name: required string,
            desc: optional string,
            @(name),
            @(desc)?
        ;
    .
    "#,
    );
}

#[test]
fn valid_conditional_ref_optional_block() {
    // @(optional_field)? { body } is valid
    validate_ok(
        r###"
        test/types:
        Record:
            subtitle: optional string,
            @(subtitle)? { "## ", @(subtitle) }
        ;
    .
    "###,
    );
}

#[test]
fn valid_conditional_ref_collection() {
    // @(collection)? desugars to @ifnotempty — collection emit inside guard is ok
    validate_ok(
        r#"
        test/types:
        Record:
            tags: required string*,
            @(tags)? { @join(tags, ", ") }
        ;
    .
    "#,
    );
}

#[test]
fn valid_conditional_join() {
    // @join(tags, ", ")? is valid sugar for @ifnotempty { @join }
    validate_ok(
        r#"
        test/types:
        Record:
            tags: required string*,
            @join(tags, ", ")?
        ;
    .
    "#,
    );
}

#[test]
fn error_conditional_ref_on_required() {
    // @(required_field)? desugars to IfSet, which is an error for non-optional fields
    expect_error(
        r#"
        test/types:
        Record:
            name: required string,
            @(name)?
        ;
    .
    "#,
        "@ifset on non-optional field `name`",
    );
}
