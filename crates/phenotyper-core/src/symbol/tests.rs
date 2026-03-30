// SPDX-License-Identifier: Apache-2.0
//! Symbol table unit tests.

use super::*;
use crate::parser;

/// Helper: parse source and build symbol table, returning (table, diags).
fn build_symbols(source: &str) -> (SymbolTable, Vec<crate::diagnostic::Diagnostic>) {
    let ast = parser::parse_pht(source, "test.pht").expect("parse should succeed");
    build(&ast, "test.pht")
}

/// Helper: parse source and build symbol table, expecting zero errors.
fn build_symbols_ok(source: &str) -> SymbolTable {
    let (table, diags) = build_symbols(source);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == crate::diagnostic::Severity::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
    table
}

/// Helper: parse source and build symbol table, expecting at least one error
/// containing the given substring.
fn expect_error(source: &str, expected_substr: &str) {
    let (_, diags) = build_symbols(source);
    let has_match = diags.iter().any(|d| {
        d.severity == crate::diagnostic::Severity::Error && d.summary.contains(expected_substr)
    });
    assert!(
        has_match,
        "expected error containing `{expected_substr}`, got: {diags:#?}"
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
    let table = build_symbols_ok(&source);

    // Namespace
    assert_eq!(table.namespace, vec!["aivolution", "format", "csv"]);

    // Types registered
    assert!(table.resolve("ScalarValue").is_some());
    assert!(table.resolve("CSVFieldValue").is_some());
    assert!(table.resolve("CSVFieldValues").is_some());
    assert!(table.resolve("CSVLine").is_some());
    assert!(table.resolve("CSVLines").is_some());
    assert!(table.resolve("CSVFile").is_some());

    // Check that ScalarValue is an alias
    assert!(matches!(
        table.resolve("ScalarValue"),
        Some(Symbol::Alias(_))
    ));

    // Check that CSVFieldValues is a plural companion
    assert!(matches!(
        table.resolve("CSVFieldValues"),
        Some(Symbol::PluralCompanion { .. })
    ));

    // 3 phenotypes declared
    assert_eq!(table.types.len(), 3);

    // Check fields of CSVFile
    let csv_file = table.types.iter().find(|t| t.singular_name == "CSVFile");
    assert!(csv_file.is_some());
    let csv_file = csv_file.unwrap();
    assert_eq!(csv_file.fields.len(), 3);
    assert_eq!(csv_file.fields[0].name, "header");
    assert_eq!(csv_file.fields[1].name, "lines");
    assert_eq!(csv_file.fields[2].name, "eol");
}

#[test]
fn valid_simple_phenotype() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#;
    let table = build_symbols_ok(source);
    assert!(matches!(
        table.resolve("Record"),
        Some(Symbol::Phenotype(_))
    ));
    assert_eq!(table.types[0].fields.len(), 1);
    assert_eq!(table.types[0].fields[0].name, "name");
    assert_eq!(
        table.types[0].fields[0].requiredness,
        Requiredness::Required
    );
}

#[test]
fn valid_with_plural() {
    let source = r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#;
    let table = build_symbols_ok(source);
    assert!(matches!(table.resolve("Item"), Some(Symbol::Phenotype(_))));
    assert!(matches!(
        table.resolve("Items"),
        Some(Symbol::PluralCompanion { .. })
    ));
    assert_eq!(table.types[0].plural_name, Some("Items".to_string()));
}

#[test]
fn valid_enum() {
    let source = r#"
        namespace test/types;
        type Color: [Red, Green, Blue];
    "#;
    let table = build_symbols_ok(source);
    assert!(matches!(table.resolve("Color"), Some(Symbol::Enum(_))));
    let color = &table.enums[0];
    assert_eq!(color.members, vec!["Red", "Green", "Blue"]);
}

#[test]
fn valid_type_alias() {
    let source = r#"
        namespace test/types;
        type Name: string;
    "#;
    let table = build_symbols_ok(source);
    assert!(matches!(table.resolve("Name"), Some(Symbol::Alias(_))));
}

#[test]
fn valid_optional_field() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            desc: optional string,
            @(name)
        ;
    "#;
    let table = build_symbols_ok(source);
    assert_eq!(
        table.types[0].fields[1].requiredness,
        Requiredness::Optional
    );
}

#[test]
fn valid_primitive_type_resolution() {
    let source = r#"
        namespace test/types;
        Record:
            s: required string,
            i: required int64,
            r: required real64,
            b: required bool,
            d: required date,
            t: required time,
            dt: required datetime,
            @(s)
        ;
    "#;
    // All primitive types should resolve without errors
    build_symbols_ok(source);
}

#[test]
fn valid_self_referencing_type() {
    let source = r#"
        namespace test/types;
        type Value: string;
        Record:
            value: required Value,
            @(value)
        ;
    "#;
    build_symbols_ok(source);
}

#[test]
fn valid_enum_as_field_type() {
    let source = r#"
        namespace test/types;
        type Color: [Red, Green, Blue];
        Tag:
            color: required Color,
            @(color)
        ;
    "#;
    build_symbols_ok(source);
}

#[test]
fn valid_plural_as_field_type() {
    let source = r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
        Container:
            items: required Items,
            @(items)
        ;
    "#;
    build_symbols_ok(source);
}

#[test]
fn valid_multiple_render_exprs() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            eol: required string,
            "prefix: ",
            @(name),
            @eol(eol),
            @eol
        ;
    "#;
    build_symbols_ok(source);
}

#[test]
fn valid_block_directive() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @ifset(footer) { @(footer) }
        ;
    "#;
    build_symbols_ok(source);
}

// ─── Duplicate detection ────────────────────────────────────────────────────

#[test]
fn error_duplicate_singular_name() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
        Record:
            value: required string,
            @(value)
        ;
    "#;
    expect_error(source, "duplicate type name `Record`");
}

#[test]
fn error_duplicate_plural_name() {
    let source = r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
        Thing plural Items:
            value: required string,
            @(value)
        ;
    "#;
    expect_error(source, "duplicate type name `Items`");
}

#[test]
fn error_singular_plural_collision() {
    let source = r#"
        namespace test/types;
        Items:
            value: required string,
            @(value)
        ;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#;
    expect_error(source, "duplicate type name `Items`");
}

#[test]
fn error_singular_equals_plural() {
    let source = r#"
        namespace test/types;
        Item plural Item:
            value: required string,
            @(value)
        ;
    "#;
    expect_error(
        source,
        "singular name `Item` and plural name `Item` must be different",
    );
}

#[test]
fn error_duplicate_field() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            name: optional string,
            @(name)
        ;
    "#;
    expect_error(source, "duplicate field `name` in type `Record`");
}

#[test]
fn error_duplicate_enum_member() {
    let source = r#"
        namespace test/types;
        type Color: [Red, Green, Red];
    "#;
    expect_error(source, "duplicate enum member `Red` in enum `Color`");
}

#[test]
fn error_duplicate_type_decl_and_def() {
    let source = r#"
        namespace test/types;
        type Record: string;
        Record:
            name: required string,
            @(name)
        ;
    "#;
    expect_error(source, "duplicate type name `Record`");
}

#[test]
fn error_duplicate_enum_and_phenotype() {
    let source = r#"
        namespace test/types;
        type Color: [Red, Green, Blue];
        Color:
            value: required string,
            @(value)
        ;
    "#;
    expect_error(source, "duplicate type name `Color`");
}

// ─── Unknown references ─────────────────────────────────────────────────────

#[test]
fn error_unknown_type_reference() {
    let source = r#"
        namespace test/types;
        Record:
            value: required UnknownType,
            @(value)
        ;
    "#;
    expect_error(source, "unknown type `UnknownType`");
}

#[test]
fn error_unknown_field_reference() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            @(nonexistent)
        ;
    "#;
    expect_error(source, "unknown field `nonexistent` in type `Record`");
}

#[test]
fn error_unknown_field_in_directive() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            @join(missing_field, name)
        ;
    "#;
    expect_error(source, "unknown field `missing_field` in type `Record`");
}

#[test]
fn error_unknown_field_in_block_directive() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            footer: optional string,
            @ifset(footer) { @(bad_field) }
        ;
    "#;
    expect_error(source, "unknown field `bad_field` in type `Record`");
}

#[test]
fn error_unknown_type_in_union() {
    let source = r#"
        namespace test/types;
        Record:
            value: required {string, UnknownType},
            @(value)
        ;
    "#;
    expect_error(source, "unknown type `UnknownType`");
}

#[test]
fn error_unknown_type_in_alias() {
    let source = r#"
        namespace test/types;
        type MyAlias: UnknownType;
    "#;
    expect_error(source, "unknown type `UnknownType`");
}

// ─── Symbol table queries ───────────────────────────────────────────────────

#[test]
fn resolve_field_by_name() {
    let source = r#"
        namespace test/types;
        Record:
            name: required string,
            age: optional int64,
            @(name)
        ;
    "#;
    let table = build_symbols_ok(source);
    let type_id = match table.resolve("Record") {
        Some(Symbol::Phenotype(id)) => *id,
        _ => panic!("Record should be a phenotype"),
    };
    assert!(table.resolve_field(type_id, "name").is_some());
    assert!(table.resolve_field(type_id, "age").is_some());
    assert!(table.resolve_field(type_id, "missing").is_none());
}

#[test]
fn plural_companion_maps_to_singular() {
    let source = r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#;
    let table = build_symbols_ok(source);

    let singular_id = match table.resolve("Item") {
        Some(Symbol::Phenotype(id)) => *id,
        _ => panic!("Item should be a phenotype"),
    };
    let plural_id = match table.resolve("Items") {
        Some(Symbol::PluralCompanion { type_id }) => *type_id,
        _ => panic!("Items should be a plural companion"),
    };
    assert_eq!(singular_id, plural_id);
}
