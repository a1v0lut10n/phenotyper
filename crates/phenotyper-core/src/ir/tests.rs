// SPDX-License-Identifier: Apache-2.0
//! IR lowering unit tests.

use super::*;
use crate::parser;
use crate::symbol::{self, Cardinality, FieldId, PrimitiveType, Requiredness};

/// Helper: parse + build symbols + lower to IR, expecting zero errors.
fn lower_ok(source: &str) -> PhenotypeModule {
    let ast = parser::parse_pht(source, "test.pht").expect("parse failed");
    let (table, sym_diags) = symbol::build(&ast, "test.pht");
    let sym_errors: Vec<_> = sym_diags
        .iter()
        .filter(|d| d.severity == crate::diagnostic::Severity::Error)
        .collect();
    assert!(sym_errors.is_empty(), "symbol errors: {sym_errors:#?}");

    let (module, ir_diags) = lower(&ast, &table, "test.pht");
    let ir_errors: Vec<_> = ir_diags
        .iter()
        .filter(|d| d.severity == crate::diagnostic::Severity::Error)
        .collect();
    assert!(ir_errors.is_empty(), "IR errors: {ir_errors:#?}");
    module
}

// ─── Module structure ───────────────────────────────────────────────────────

#[test]
fn lower_csv_fixture() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/csv_basic.pht"
    ))
    .expect("fixture file missing");
    let module = lower_ok(&source);

    assert_eq!(module.namespace, "aivolution/format/csv");
    assert_eq!(module.types.len(), 3);
    assert_eq!(module.enums.len(), 0);
    assert_eq!(module.aliases.len(), 1);

    // ScalarValue alias
    assert_eq!(module.aliases[0].name, "ScalarValue");
    assert!(matches!(module.aliases[0].target, ValueType::Union(_)));

    // CSVFieldValue
    let fv = &module.types[0];
    assert_eq!(fv.singular_name, "CSVFieldValue");
    assert_eq!(fv.plural_name, Some("CSVFieldValues".to_string()));
    assert_eq!(fv.fields.len(), 1);

    // CSVLine
    let line = &module.types[1];
    assert_eq!(line.singular_name, "CSVLine");
    assert_eq!(line.plural_name, Some("CSVLines".to_string()));

    // CSVFile
    let csv_file = &module.types[2];
    assert_eq!(csv_file.singular_name, "CSVFile");
    assert_eq!(csv_file.plural_name, None);
    assert_eq!(csv_file.fields.len(), 3);
}

// ─── Namespace ──────────────────────────────────────────────────────────────

#[test]
fn namespace_is_joined() {
    let source = "aivolution/format/csv: .";
    // This fails at symbol level (no types) but we can still test
    let ast = parser::parse_pht(source, "test.pht").unwrap();
    let (table, _) = symbol::build(&ast, "test.pht");
    let (module, _) = lower(&ast, &table, "test.pht");
    assert_eq!(module.namespace, "aivolution/format/csv");
}

// ─── Primitive type resolution ──────────────────────────────────────────────

#[test]
fn primitives_resolve_correctly() {
    let source = r#"
        test/types:
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
    .
    "#;
    let module = lower_ok(source);
    let fields = &module.types[0].fields;

    assert_eq!(fields[0].ty, ValueType::Primitive(PrimitiveType::String));
    assert_eq!(fields[1].ty, ValueType::Primitive(PrimitiveType::Int64));
    assert_eq!(fields[2].ty, ValueType::Primitive(PrimitiveType::Real64));
    assert_eq!(fields[3].ty, ValueType::Primitive(PrimitiveType::Bool));
    assert_eq!(fields[4].ty, ValueType::Primitive(PrimitiveType::Date));
    assert_eq!(fields[5].ty, ValueType::Primitive(PrimitiveType::Time));
    assert_eq!(fields[6].ty, ValueType::Primitive(PrimitiveType::DateTime));
}

// ─── Cardinality normalization ──────────────────────────────────────────────

#[test]
fn cardinality_plus() {
    let source = r#"
        test/types:
        Record:
            items: required string+,
            @(items)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].fields[0].cardinality,
        Cardinality::OneOrMore
    );
}

#[test]
fn cardinality_star() {
    let source = r#"
        test/types:
        Record:
            items: required string*,
            @(items)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].fields[0].cardinality,
        Cardinality::ZeroOrMore
    );
}

#[test]
fn cardinality_default_one() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            @(name)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.types[0].fields[0].cardinality, Cardinality::One);
}

// ─── Requiredness ───────────────────────────────────────────────────────────

#[test]
fn required_and_optional_fields() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            desc: optional string,
            @(name)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].fields[0].requiredness,
        Requiredness::Required
    );
    assert_eq!(
        module.types[0].fields[1].requiredness,
        Requiredness::Optional
    );
}

// ─── Singular/plural normalization ──────────────────────────────────────────

#[test]
fn plural_companion_lowers_to_user_plural() {
    let source = r#"
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
    "#;
    let module = lower_ok(source);

    // Item type
    assert_eq!(module.types[0].singular_name, "Item");
    assert_eq!(module.types[0].plural_name, Some("Items".to_string()));

    // Container.items should be UserPlural
    let items_field = &module.types[1].fields[0];
    assert!(matches!(items_field.ty, ValueType::UserPlural { .. }));
    if let ValueType::UserPlural { collection_of } = items_field.ty {
        assert_eq!(collection_of, module.types[0].id);
    }
}

#[test]
fn singular_ref_lowers_to_user_singular() {
    let source = r#"
        test/types:
        Item plural Items:
            value: required string,
            @(value)
        ;
        Wrapper:
            item: required Item,
            @(item)
        ;
    .
    "#;
    let module = lower_ok(source);
    let item_field = &module.types[1].fields[0];
    assert!(matches!(item_field.ty, ValueType::UserSingular(_)));
}

// ─── Enum type resolution ───────────────────────────────────────────────────

#[test]
fn enum_type_lowered() {
    let source = r#"
        test/types:
        type Color: [Red, Green, Blue];
        Tag:
            color: required Color,
            @(color)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.enums.len(), 1);
    assert_eq!(module.enums[0].name, "Color");
    assert_eq!(module.enums[0].members, vec!["Red", "Green", "Blue"]);

    // Field type should be Enum
    let field = &module.types[0].fields[0];
    assert!(matches!(field.ty, ValueType::Enum(_)));
}

// ─── Type alias lowering ────────────────────────────────────────────────────

#[test]
fn alias_lowers_to_type_alias() {
    let source = r#"
        test/types:
        type Name: string;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.aliases.len(), 1);
    assert_eq!(module.aliases[0].name, "Name");
    assert_eq!(
        module.aliases[0].target,
        ValueType::Primitive(PrimitiveType::String)
    );
}

// ─── Union type lowering ────────────────────────────────────────────────────

#[test]
fn union_flattened() {
    let source = r#"
        test/types:
        type ScalarValue: {int64, real64, string};
    .
    "#;
    let module = lower_ok(source);
    let target = &module.aliases[0].target;
    if let ValueType::Union(members) = target {
        assert_eq!(
            members,
            &[
                ValueType::Primitive(PrimitiveType::Int64),
                ValueType::Primitive(PrimitiveType::Real64),
                ValueType::Primitive(PrimitiveType::String),
            ]
        );
    } else {
        panic!("expected Union, got {target:?}");
    }
}

// ─── Directive canonicalization ─────────────────────────────────────────────

#[test]
fn field_ref_becomes_emit() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            @(name)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.types[0].render, vec![RenderNode::Emit(FieldId(0))]);
}

#[test]
fn string_literal_becomes_text() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            "Hello, ",
            @(name)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::Text("Hello, ".to_string())
    );
}

#[test]
fn string_literal_unescape() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            "line1\nline2",
            @(name)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::Text("line1\nline2".to_string())
    );
}

// ─── Eol lowering ───────────────────────────────────────────────────────────

#[test]
fn bare_eol_becomes_eol_none() {
    let source = r#"
        test/types:
        Record:
            value: required string,
            @(value),
            @eol
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.types[0].render[1], RenderNode::Eol { field: None });
}

#[test]
fn eol_with_arg_becomes_eol_some() {
    let source = r#"
        test/types:
        Record:
            value: required string,
            eol: required string,
            @(value),
            @eol(eol)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[1],
        RenderNode::Eol {
            field: Some(FieldId(1))
        }
    );
}

#[test]
fn eol_empty_parens_becomes_eol_none() {
    let source = r#"
        test/types:
        Record:
            value: required string,
            @(value),
            @eol()
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(module.types[0].render[1], RenderNode::Eol { field: None });
}

// ─── Join lowering ──────────────────────────────────────────────────────────

#[test]
fn join_with_field_separator() {
    let source = r#"
        test/types:
        Record:
            fields: required string,
            separator: required string,
            @join(fields, separator)
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::Join {
            field: FieldId(0),
            separator: SeparatorExpr::Field(FieldId(1)),
        }
    );
}

#[test]
fn join_with_literal_separator() {
    let source = r#"
        test/types:
        Record:
            tags: required string,
            @join(tags, ", ")
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::Join {
            field: FieldId(0),
            separator: SeparatorExpr::Literal(", ".to_string()),
        }
    );
}

// ─── Block directive lowering ───────────────────────────────────────────────

#[test]
fn ifset_lowered() {
    let source = r#"
        test/types:
        Record:
            name: required string,
            footer: optional string,
            @(name),
            @ifset(footer) { @(footer) }
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[1],
        RenderNode::IfSet {
            field: FieldId(1),
            body: vec![RenderNode::Emit(FieldId(1))],
        }
    );
}

#[test]
fn ifnotempty_lowered() {
    let source = r#"
        test/types:
        Record:
            tags: required string*,
            @ifnotempty(tags) { @join(tags, ", ") }
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::IfNotEmpty {
            field: FieldId(0),
            body: vec![RenderNode::Join {
                field: FieldId(0),
                separator: SeparatorExpr::Literal(", ".to_string()),
            }],
        }
    );
}

#[test]
fn nested_block_body() {
    let source = r#"
        test/types:
        Record:
            footer: optional string,
            @ifset(footer) { @eol, @(footer) }
        ;
    .
    "#;
    let module = lower_ok(source);
    assert_eq!(
        module.types[0].render[0],
        RenderNode::IfSet {
            field: FieldId(0),
            body: vec![
                RenderNode::Eol { field: None },
                RenderNode::Emit(FieldId(0)),
            ],
        }
    );
}

// ─── Full CSV fixture IR validation ─────────────────────────────────────────

#[test]
fn csv_fixture_render_nodes() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/valid/csv_basic.pht"
    ))
    .expect("fixture file missing");
    let module = lower_ok(&source);

    // CSVFieldValue: @(value)
    let fv = &module.types[0];
    assert_eq!(fv.render, vec![RenderNode::Emit(FieldId(0))]);

    // CSVLine: @join(fields, separator)
    let line = &module.types[1];
    assert_eq!(
        line.render,
        vec![RenderNode::Join {
            field: FieldId(0),
            separator: SeparatorExpr::Field(FieldId(1)),
        }]
    );

    // CSVFile: @(header), @eol(eol), @join(lines, eol)
    let csv_file = &module.types[2];
    assert_eq!(csv_file.render.len(), 3);
    assert_eq!(csv_file.render[0], RenderNode::Emit(FieldId(0)));
    assert_eq!(
        csv_file.render[1],
        RenderNode::Eol {
            field: Some(FieldId(2))
        }
    );
    assert_eq!(
        csv_file.render[2],
        RenderNode::Join {
            field: FieldId(1),
            separator: SeparatorExpr::Field(FieldId(2)),
        }
    );
}
