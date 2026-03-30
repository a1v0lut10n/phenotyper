// SPDX-License-Identifier: Apache-2.0
//! Parser unit tests — validates parsing of Phenotyper v1 syntax.

use super::*;

#[test]
fn parse_namespace_only() {
    let source = "namespace aivolution/format/csv;";
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_namespace_and_uses() {
    let source = r#"
        namespace aivolution/format/csv;
        uses aivolution/core/types;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_type_alias() {
    let source = r#"
        namespace test/types;
        type CSVFieldValue: string;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_enum_decl() {
    let source = r#"
        namespace test/types;
        type Encoding: [UTF8, ASCII, Latin1];
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_simple_phenotype() {
    let source = r#"
        namespace test/format;
        CSVLine:
            fields: required string,
            separator: required string,
            @(fields)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_phenotype_with_plural() {
    let source = r#"
        namespace test/format;
        CSVLine plural CSVLines:
            fields: required string,
            separator: required string,
            @(fields)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_directive_call() {
    let source = r#"
        namespace test/format;
        CSVLine:
            fields: required string,
            separator: required string,
            @join(fields, separator)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_bare_eol() {
    let source = r#"
        namespace test/format;
        Record:
            value: required string,
            @(value),
            @eol
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_eol_with_field() {
    let source = r#"
        namespace test/format;
        Record:
            value: required string,
            eol: required string,
            @(value),
            @eol(eol)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_block_directive() {
    let source = r#"
        namespace test/format;
        CSVFile:
            footer: optional string,
            @ifset(footer) { @eol, @(footer) }
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_cardinality_plus() {
    let source = r#"
        namespace test/format;
        CSVFile:
            lines: required string+,
            @(lines)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_cardinality_star() {
    let source = r#"
        namespace test/format;
        CSVFile:
            lines: required string*,
            @(lines)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_optional_field() {
    let source = r#"
        namespace test/format;
        CSVFile:
            header: optional string,
            @(header)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_string_literal_render() {
    let source = r#"
        namespace test/format;
        Record:
            name: required string,
            "Hello, ",
            @(name)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_union_type() {
    let source = r#"
        namespace test/types;
        Flexible:
            value: required {string, int64},
            @(value)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_line_comments() {
    let source = r#"
        // This is the main namespace
        namespace test/format;
        // A record type
        Record:
            name: required string, // field name
            @(name)
        ;
    "#;
    let result = parse_pht(source, "test.pht");
    assert!(result.is_ok(), "parse failed: {result:?}");
}

#[test]
fn parse_block_comments() {
    let source = r#"
        /* Multi-line
           comment */
        namespace test/format;
        Record:
            name: required string,
            @(name)
        ;
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
namespace test/format;
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

// --- Negative tests ---

#[test]
fn parse_missing_semicolon() {
    let source = "namespace test/format";
    let result = parse_pht(source, "test.pht");
    assert!(result.is_err());
}

#[test]
fn parse_empty_input() {
    let source = "";
    let result = parse_pht(source, "test.pht");
    // Empty input should fail (missing namespace)
    assert!(result.is_err());
}
