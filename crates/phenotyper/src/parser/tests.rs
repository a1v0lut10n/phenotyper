// SPDX-License-Identifier: Apache-2.0
//! Parser unit tests — validates parsing of Phenotyper v2 syntax.

use super::*;

#[test]
fn parse_namespace_only() {
    let source = "test/ns: .";
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_namespace_and_uses() {
    let source = r#"
        aivolution/format/csv:
        uses aivolution/core/types;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_type_alias() {
    let source = r#"
        test/types:
        type CSVFieldValue: string;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_enum_decl() {
    let source = r#"
        test/types:
        type Encoding: [UTF8, ASCII, Latin1];
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_simple_phenotype() {
    let source = r#"
        test/format:
        CSVLine:
            fields: required string,
            separator: required string,
            @(fields)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_phenotype_with_plural() {
    let source = r#"
        test/format:
        CSVLine plural CSVLines:
            fields: required string,
            separator: required string,
            @(fields)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_directive_call() {
    let source = r#"
        test/format:
        CSVLine:
            fields: required string,
            separator: required string,
            @join(fields, separator)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_bare_eol() {
    let source = r#"
        test/format:
        Record:
            value: required string,
            @(value),
            @eol
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_eol_with_field() {
    let source = r#"
        test/format:
        Record:
            value: required string,
            eol: required string,
            @(value),
            @eol(eol)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_block_directive() {
    let source = r#"
        test/format:
        CSVFile:
            footer: optional string,
            @ifset(footer) { @eol, @(footer) }
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_cardinality_plus() {
    let source = r#"
        test/format:
        CSVFile:
            lines: required string+,
            @(lines)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_cardinality_star() {
    let source = r#"
        test/format:
        CSVFile:
            lines: required string*,
            @(lines)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_optional_field() {
    let source = r#"
        test/format:
        CSVFile:
            header: optional string,
            @(header)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_string_literal_render() {
    let source = r#"
        test/format:
        Record:
            name: required string,
            "Hello, ",
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_union_type() {
    let source = r#"
        test/types:
        Flexible:
            value: required {string, int64},
            @(value)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_line_comments() {
    let source = r#"
        // This is the main namespace
        test/format:
        // A record type
        Record:
            name: required string, // field name
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_block_comments() {
    let source = r#"
        /* Multi-line
           comment */
        test/format:
        Record:
            name: required string,
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_csv_fixture() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/csv_basic.pht"
    ))
    .expect("fixture file missing");
    let result = parse_pht(&source, "csv_basic.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_optional_fixture() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/optional_and_collections.pht"
    ))
    .expect("fixture file missing");
    let result = parse_pht(&source, "optional_and_collections.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_md_extraction() {
    let md = r#"# Example

```pht
test/format:
Record:
    name: required string,
    @(name)
;
```

Some text here.
"#;
    let result = parse_md(md, "example.md");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

// --- v2-specific tests ---

#[test]
fn parse_single_segment_namespace() {
    // GLR: single-segment namespace should be disambiguated from phenotype
    let source = "csv: .";
    let result = parse_pht(source, "test.pht");
    assert!(
        result.is_ok(),
        "single-segment namespace parse failed: {result:?}"
    );
}

#[test]
fn parse_empty_namespace() {
    let source = "test/empty: .";
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "empty namespace parse failed: {result:?}");
}

#[test]
fn parse_md_without_trailing_dot() {
    // The parser should auto-append '.' for .md containers
    let md = r#"# Example

```pht
test/format:
Record:
    name: required string,
    @(name)
;
```
"#;
    let result = parse_md(md, "example.md");
    assert!(
        result.is_ok(),
        "md without trailing dot should parse: {result:?}"
    );
}

// --- ? suffix operator tests (v2) ---

#[test]
fn parse_conditional_ref_bare() {
    let source = r#"
        test/format:
        Record:
            subtitle: optional string,
            @(subtitle)?
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "bare conditional ref failed: {result:?}");
}

#[test]
fn parse_conditional_ref_block() {
    let source = r###"
        test/format:
        Record:
            subtitle: optional string,
            @(subtitle)? { "## ", @(subtitle) }
        ;
    .
    "###;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "block conditional ref failed: {result:?}");
}

#[test]
fn parse_conditional_join_bare() {
    let source = r#"
        test/format:
        Record:
            tags: required string*,
            @join(tags, ", ")?
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "bare conditional join failed: {result:?}");
}

#[test]
fn parse_conditional_join_block() {
    let source = r#"
        test/format:
        Record:
            tags: required string*,
            @join(tags, ", ")? { "Tags: ", @join(tags, ", ") }
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "block conditional join failed: {result:?}");
}

#[test]
fn parse_conditional_ref_on_required() {
    // ? on required should parse (semantic pass warns)
    let source = r#"
        test/format:
        Record:
            name: required string,
            @(name)?
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "? on required should parse: {result:?}");
}

// --- Nested phenotype tests (v2) ---

#[test]
fn parse_nested_phenotype() {
    let source = r#"
        test/types:
        Parent:
            name: required string,
            Child:
                value: required string,
                @(value)
            ;,
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "nested phenotype failed: {result:?}");
}

#[test]
fn parse_nested_two_levels() {
    let source = r#"
        test/types:
        GrandParent:
            name: required string,
            Parent:
                value: required string,
                Child:
                    count: required int64,
                    @(count)
                ;,
                @(value)
            ;,
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "two-level nesting failed: {result:?}");
}

#[test]
fn parse_scoped_field_ref() {
    let source = r#"
        test/types:
        Parent:
            name: required string,
            Child:
                value: required string,
                @(Parent/name), @(value)
            ;,
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "scoped field ref failed: {result:?}");
}

#[test]
fn parse_nested_with_plural() {
    let source = r#"
        test/types:
        Container:
            name: required string,
            Item plural Items:
                value: required string,
                @(value)
            ;,
            @(name)
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "nested with plural failed: {result:?}");
}

#[test]
fn parse_nested_mixed_body() {
    // Fields, nested types, and render exprs interleaved
    let source = r#"
        test/types:
        Report:
            title: required string,
            Section plural Sections:
                heading: required string,
                @(heading)
            ;,
            footer: optional string,
            @(title),
            @(footer)? { @(footer) }
        ;
    .
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "mixed body failed: {result:?}");
}

// --- Negative tests ---

#[test]
fn parse_missing_dot_terminator() {
    // v2 requires '.' to terminate namespace scope
    let source = "test/format:
        Record:
            name: required string,
            @(name)
        ;
    ";
    let result = parse_pht(source, "test.pht");
    assert!(result.is_err(), "should fail without '.' terminator");
}

#[test]
fn parse_empty_input() {
    let source = "";
    let result = parse_pht(source, "test.pht");
    // Empty input should fail (missing namespace scope)
    assert!(result.is_err());
}
