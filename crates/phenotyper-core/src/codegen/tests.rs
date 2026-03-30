// SPDX-License-Identifier: Apache-2.0
//! Code generation tests.

use crate::codegen;
use crate::ir;
use crate::parser;
use crate::symbol;

/// Helper: parse → symbols → IR → codegen, expecting zero errors.
fn generate_ok(source: &str) -> String {
    let ast = parser::parse_pht(source, "test.pht").expect("parse failed");
    let (table, sym_diags) = symbol::build(&ast, "test.pht");
    let sym_errors: Vec<_> = sym_diags
        .iter()
        .filter(|d| d.severity == crate::diagnostic::Severity::Error)
        .collect();
    assert!(sym_errors.is_empty(), "symbol errors: {sym_errors:#?}");

    let (module, ir_diags) = ir::lower(&ast, &table, "test.pht");
    let ir_errors: Vec<_> = ir_diags
        .iter()
        .filter(|d| d.severity == crate::diagnostic::Severity::Error)
        .collect();
    assert!(ir_errors.is_empty(), "IR errors: {ir_errors:#?}");

    codegen::generate(&module)
}

// ─── Module structure ───────────────────────────────────────────────────────

#[test]
fn generated_header() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("@generated"));
    assert!(code.contains("#![allow(dead_code"));
}

#[test]
fn render_trait_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("pub trait Render"));
    assert!(code.contains("fn render_into(&self, out: &mut String)"));
    assert!(code.contains("fn render(&self) -> String"));
}

#[test]
fn build_error_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("pub enum BuildError"));
    assert!(code.contains("MissingField"));
    assert!(code.contains("CardinalityViolation"));
    assert!(code.contains("impl std::error::Error for BuildError"));
}

// ─── Singular struct ────────────────────────────────────────────────────────

#[test]
fn singular_struct_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            count: required int64,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("pub struct Record"));
    assert!(code.contains("pub name: String"));
    assert!(code.contains("pub count: i64"));
}

#[test]
fn optional_field_is_option() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            desc: optional string,
            @(name),
            @ifset(desc) { @(desc) }
        ;
    "#,
    );
    assert!(code.contains("pub desc: Option<String>"));
}

#[test]
fn collection_field_is_vec() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            tags: required string*,
            @join(tags, ", ")
        ;
    "#,
    );
    assert!(code.contains("pub tags: Vec<String>"));
}

// ─── Builder ────────────────────────────────────────────────────────────────

#[test]
fn builder_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("pub struct RecordBuilder"));
    assert!(code.contains("impl RecordBuilder"));
    assert!(code.contains("pub fn new()"));
    assert!(code.contains("pub fn name(&mut self"));
    assert!(code.contains("pub fn build(self) -> Result<Record, BuildError>"));
}

#[test]
fn builder_convenience_method() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("impl Record"));
    assert!(code.contains("pub fn builder() -> RecordBuilder"));
}

#[test]
fn builder_validates_missing_required() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("MissingField(\"name\")"));
}

#[test]
fn builder_validates_one_or_more() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            items: required string+,
            @join(items, ", ")
        ;
    "#,
    );
    assert!(code.contains("CardinalityViolation(\"items\")"));
}

// ─── Render impl ────────────────────────────────────────────────────────────

#[test]
fn render_impl_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("impl Render for Record"));
    assert!(code.contains("fn render_into(&self, out: &mut String)"));
}

#[test]
fn render_string_field() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name)
        ;
    "#,
    );
    assert!(code.contains("out.push_str(&self.name)"));
}

#[test]
fn render_text_literal() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            "Hello: ",
            @(name)
        ;
    "#,
    );
    assert!(code.contains("out.push_str(\"Hello: \")"));
}

#[test]
fn render_eol_bare() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            @(name),
            @eol
        ;
    "#,
    );
    assert!(code.contains("out.push('\\n')"));
}

#[test]
fn render_eol_with_field() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            eol: required string,
            @(name),
            @eol(eol)
        ;
    "#,
    );
    assert!(code.contains("out.push_str(&self.eol)"));
}

#[test]
fn render_join_literal_separator() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            tags: required string*,
            @join(tags, ", ")
        ;
    "#,
    );
    assert!(code.contains("out.push_str(\", \")"));
    assert!(code.contains("self.tags.iter()"));
}

#[test]
fn render_join_field_separator() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            items: required string*,
            sep: required string,
            @join(items, sep)
        ;
    "#,
    );
    assert!(code.contains("out.push_str(&self.sep)"));
}

#[test]
fn render_ifset() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @ifset(footer) { @(footer) }
        ;
    "#,
    );
    assert!(code.contains("if let Some(ref val) = self.footer"));
}

#[test]
fn render_ifnotempty() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Record:
            tags: required string*,
            @ifnotempty(tags) { @join(tags, ", ") }
        ;
    "#,
    );
    assert!(code.contains("if !self.tags.is_empty()"));
}

// ─── Plural wrapper ─────────────────────────────────────────────────────────

#[test]
fn plural_wrapper_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#,
    );
    assert!(code.contains("pub struct Items"));
    assert!(code.contains("items: Vec<Item>"));
    assert!(code.contains("pub fn new()"));
    assert!(code.contains("pub fn from_vec("));
    assert!(code.contains("pub fn into_vec("));
    assert!(code.contains("pub fn as_slice("));
    assert!(code.contains("pub fn iter("));
}

#[test]
fn plural_from_into_vec() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#,
    );
    assert!(code.contains("impl From<Vec<Item>> for Items"));
    assert!(code.contains("impl From<Items> for Vec<Item>"));
}

#[test]
fn plural_builder_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#,
    );
    assert!(code.contains("pub struct ItemsBuilder"));
    assert!(code.contains("pub fn push(&mut self, item: Item)"));
    assert!(code.contains("pub fn extend(&mut self"));
}

#[test]
fn plural_render_impl() {
    let code = generate_ok(
        r#"
        namespace test/types;
        Item plural Items:
            value: required string,
            @(value)
        ;
    "#,
    );
    assert!(code.contains("impl Render for Items"));
}

// ─── Enum types ─────────────────────────────────────────────────────────────

#[test]
fn enum_type_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        type Color: [Red, Green, Blue];
    "#,
    );
    assert!(code.contains("pub enum Color"));
    assert!(code.contains("Red,"));
    assert!(code.contains("Green,"));
    assert!(code.contains("Blue,"));
}

#[test]
fn enum_render_original_spelling() {
    let code = generate_ok(
        r#"
        namespace test/types;
        type Visibility: [public, protected, private];
    "#,
    );
    // Variants should be PascalCase
    assert!(code.contains("Public,"));
    assert!(code.contains("Protected,"));
    assert!(code.contains("Private,"));
    // Render should use original spelling
    assert!(code.contains("out.push_str(\"public\")"));
    assert!(code.contains("out.push_str(\"protected\")"));
    assert!(code.contains("out.push_str(\"private\")"));
}

// ─── Union/alias types ──────────────────────────────────────────────────────

#[test]
fn union_alias_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        type ScalarValue: {int64, real64, string};
    "#,
    );
    assert!(code.contains("pub enum ScalarValue"));
    assert!(code.contains("Int64(i64)"));
    assert!(code.contains("Real64(f64)"));
    assert!(code.contains("String(String)"));
}

#[test]
fn simple_alias_generated() {
    let code = generate_ok(
        r#"
        namespace test/types;
        type Name: string;
    "#,
    );
    assert!(code.contains("pub type Name = String;"));
}

// ─── Naming strategy ────────────────────────────────────────────────────────

#[test]
fn naming_csv_types() {
    let code = generate_ok(
        r#"
        namespace test/types;
        CSVLine plural CSVLines:
            value: required string,
            @(value)
        ;
    "#,
    );
    assert!(code.contains("pub struct CsvLine"));
    assert!(code.contains("pub struct CsvLines"));
    assert!(code.contains("pub struct CsvLineBuilder"));
    assert!(code.contains("pub struct CsvLinesBuilder"));
}

// ─── CSV fixture ────────────────────────────────────────────────────────────

#[test]
fn csv_fixture_compiles() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/csv_basic.pht"
    ))
    .expect("fixture file missing");
    let code = generate_ok(&source);

    // Check key structures are present
    assert!(code.contains("pub struct CsvFieldValue"));
    assert!(code.contains("pub struct CsvFieldValues"));
    assert!(code.contains("pub struct CsvLine"));
    assert!(code.contains("pub struct CsvLines"));
    assert!(code.contains("pub struct CsvFile"));
    assert!(code.contains("pub enum ScalarValue"));

    // Check builders
    assert!(code.contains("pub struct CsvFieldValueBuilder"));
    assert!(code.contains("pub struct CsvLineBuilder"));
    assert!(code.contains("pub struct CsvFileBuilder"));

    // Check Render impls
    assert!(code.contains("impl Render for CsvFieldValue"));
    assert!(code.contains("impl Render for CsvLine"));
    assert!(code.contains("impl Render for CsvFile"));
    assert!(code.contains("impl Render for CsvFieldValues"));
    assert!(code.contains("impl Render for CsvLines"));
}
