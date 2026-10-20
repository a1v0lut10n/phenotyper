// SPDX-License-Identifier: Apache-2.0
//! Semantic validation pass over the IR.
//!
//! Implements REQ-COMP-009 validation phases:
//! - Type validation (plural integrity, union member validity)
//! - Render validation (directive constraints, field compatibility)
//! - Generation validation (cyclic type detection)

use std::collections::HashSet;

use crate::diagnostic::Diagnostic;
use crate::ir::{FieldDef, PhenotypeModule, PhenotypeType, RenderNode, SeparatorExpr, ValueType};
use crate::symbol::{Cardinality, FieldId, Requiredness, TypeId};

/// Validate the entire module.
pub fn validate_module(module: &PhenotypeModule, file: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for pt in &module.types {
        validate_type(pt, module, file, &mut diags);
    }

    validate_cyclic_types(module, file, &mut diags);

    diags
}

/// Validate a single phenotype type.
fn validate_type(
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    // Validate render is non-empty
    if pt.render.is_empty() {
        diags.push(super::error(
            file,
            format!("type `{}` has no render expressions", pt.singular_name),
        ));
    }

    // Imported types inside unions are not supported yet (PHT-0027 keeps
    // union variant naming local); reject them honestly rather than let
    // codegen emit an unusable variant.
    for field in &pt.fields {
        if let ValueType::Union(members) = &field.ty {
            if members.iter().any(|m| matches!(m, ValueType::Imported(_))) {
                diags.push(super::error(
                    file,
                    format!(
                        "field `{}` in type `{}`: imported types are not yet supported \
                         inside unions",
                        field.name, pt.singular_name
                    ),
                ));
            }
        }
    }

    // Validate each render node
    for node in &pt.render {
        validate_render_node(node, pt, module, file, diags, false);
    }

    // PHT-0028: the annotation checks.
    validate_elasticity(pt, file, diags);
    validate_volatile_order(pt, module, file, diags);
}

/// PHT-0028: elasticity must be REALISABLE. A `drop` field must render
/// as unset does today — optional (the existing optional-outside-@ifset
/// check then guards every reference) or a collection (the join drops
/// tail items). A `truncate` field must be a scalar string — there is
/// nothing honest to cut in the middle of an int or a sub-phenotype.
fn validate_elasticity(pt: &PhenotypeType, file: &str, diags: &mut Vec<Diagnostic>) {
    for field in &pt.fields {
        match &field.annotations.elastic {
            Some(crate::ir::Elastic::Drop { .. }) => {
                let droppable = field.requiredness == Requiredness::Optional
                    || is_collection_type(&field.ty, &field.cardinality);
                if !droppable {
                    diags.push(super::error(
                        file,
                        format!(
                            "field `{}` in type `{}`: elastic(drop) needs a field that can \
                             render as unset — make it optional (and guard references \
                             with @ifset) or a collection",
                            field.name, pt.singular_name
                        ),
                    ));
                }
            }
            Some(crate::ir::Elastic::Truncate { .. })
                if !is_scalar_string(&field.ty)
                    || is_collection_type(&field.ty, &field.cardinality) =>
            {
                diags.push(super::error(
                    file,
                    format!(
                        "field `{}` in type `{}`: elastic(truncate) cuts text — it needs \
                         a scalar string field",
                        field.name, pt.singular_name
                    ),
                ));
            }
            Some(crate::ir::Elastic::Truncate { .. }) | None => {}
        }
    }
}

/// PHT-0028: stable-before-volatile, by construction. Within a type's
/// template, no STABLE field reference may follow the first volatile
/// one — a volatile byte invalidates every cached byte after it, so
/// stable content downstream of volatile content is a silent KV-reuse
/// loss (the GEN-081 field lesson, made a compile error). Literal text
/// is exempt: it belongs to whichever unit it annotates. Volatility is
/// transitive through same-module phenotype fields.
fn validate_volatile_order(
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let volatile_types = crate::ir::volatile_type_set(module);
    let is_volatile = |id: FieldId| -> bool {
        find_field(pt, id).is_some_and(|f| {
            f.annotations.volatile
                || referenced_type_id(&f.ty).is_some_and(|t| volatile_types.contains(&t))
        })
    };
    let mut first_volatile: Option<FieldId> = None;
    let mut refs: Vec<FieldId> = Vec::new();
    collect_field_refs(&pt.render, &mut refs);
    for id in refs {
        if is_volatile(id) {
            first_volatile.get_or_insert(id);
        } else if let Some(v) = first_volatile {
            diags.push(super::error(
                file,
                format!(
                    "type `{}`: stable field `{}` is referenced after volatile field `{}` —                      a volatile byte invalidates every cached byte after it; move the                      volatile reference last, or mark `{}` volatile too",
                    pt.singular_name,
                    field_id_name(pt, id),
                    field_id_name(pt, v),
                    field_id_name(pt, id),
                ),
            ));
            return; // one clear error per type beats a cascade
        }
    }
}

/// Every field reference in template order, nested bodies flattened in
/// place (an @ifset's body sits where the @ifset sits).
fn collect_field_refs(nodes: &[RenderNode], out: &mut Vec<FieldId>) {
    for node in nodes {
        match node {
            RenderNode::Text(_) | RenderNode::ParentFieldRef { .. } => {}
            RenderNode::Emit(id) => out.push(*id),
            RenderNode::Join { field, separator } => {
                out.push(*field);
                if let SeparatorExpr::Field(id) = separator {
                    out.push(*id);
                }
            }
            RenderNode::Eol { field } => {
                if let Some(id) = field {
                    out.push(*id);
                }
            }
            RenderNode::IfSet { field, body } | RenderNode::IfNotEmpty { field, body } => {
                out.push(*field);
                collect_field_refs(body, out);
            }
        }
    }
}

/// Validate a render node in context.
///
/// `inside_ifset_for` tracks whether we're inside an `@ifset` block for a
/// particular field, which relaxes the "optional field outside @ifset" check.
#[allow(clippy::only_used_in_recursion)]
fn validate_render_node(
    node: &RenderNode,
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    file: &str,
    diags: &mut Vec<Diagnostic>,
    inside_guard: bool,
) {
    match node {
        // Parent-scoped field refs are valid by construction — scope was
        // verified during IR lowering. No additional validation needed.
        RenderNode::ParentFieldRef { .. } => {}

        RenderNode::Emit(field_id) => {
            if let Some(field) = find_field(pt, *field_id) {
                // T-062: Direct @emit of plural field → error
                if is_collection_type(&field.ty, &field.cardinality) {
                    diags.push(super::error_with_suggestion(
                        file,
                        format!(
                            "cannot directly emit collection field `{}` in type `{}`",
                            field.name, pt.singular_name
                        ),
                        "collection fields must be rendered with @join".to_string(),
                        format!("use @join({}, separator) instead", field.name),
                    ));
                }

                // T-063: Direct @emit of optional fields outside @ifset → error
                if field.requiredness == Requiredness::Optional && !inside_guard {
                    diags.push(super::error_with_suggestion(
                        file,
                        format!(
                            "cannot directly emit optional field `{}` in type `{}`",
                            field.name, pt.singular_name
                        ),
                        "optional fields must be guarded by @ifset".to_string(),
                        format!("use @ifset({}) {{ @({}) }} instead", field.name, field.name),
                    ));
                }
            }
        }

        RenderNode::Join { field, separator } => {
            // T-064: Validate separator type — must be singular scalar string
            if let Some(sep_field) = match separator {
                SeparatorExpr::Field(fid) => find_field(pt, *fid),
                SeparatorExpr::Literal(_) => None,
            } {
                validate_separator_field(sep_field, pt, file, diags);
            }

            // Join field should be a collection
            if let Some(field_def) = find_field(pt, *field) {
                if !is_collection_type(&field_def.ty, &field_def.cardinality) {
                    diags.push(super::error(
                        file,
                        format!(
                            "@join field `{}` in type `{}` is not a collection",
                            field_def.name, pt.singular_name
                        ),
                    ));
                }
            }
        }

        RenderNode::Eol { field } => {
            // Eol with a field: the field should be a string
            if let Some(fid) = field {
                if let Some(field_def) = find_field(pt, *fid) {
                    if !is_scalar_string(&field_def.ty) {
                        diags.push(super::error(
                            file,
                            format!(
                                "@eol field `{}` in type `{}` must be a string type",
                                field_def.name, pt.singular_name
                            ),
                        ));
                    }
                }
            }
        }

        RenderNode::IfSet { field, body } => {
            // T-063a: @ifset on a non-optional field → error
            if let Some(field_def) = find_field(pt, *field) {
                if field_def.requiredness != Requiredness::Optional {
                    diags.push(super::error_with_explanation(
                        file,
                        format!(
                            "@ifset on non-optional field `{}` in type `{}`",
                            field_def.name, pt.singular_name
                        ),
                        "@ifset is only meaningful for optional fields".to_string(),
                    ));
                }
            }

            // Validate body (inside guard context)
            if body.is_empty() {
                diags.push(super::error(
                    file,
                    format!(
                        "@ifset block for `{}` in type `{}` has empty body",
                        field_id_name(pt, *field),
                        pt.singular_name
                    ),
                ));
            }
            for child in body {
                validate_render_node(child, pt, module, file, diags, true);
            }
        }

        RenderNode::IfNotEmpty { field, body } => {
            // T-063b: @ifnotempty on a non-collection field → error
            if let Some(field_def) = find_field(pt, *field) {
                if !is_collection_type(&field_def.ty, &field_def.cardinality) {
                    diags.push(super::error_with_explanation(
                        file,
                        format!(
                            "@ifnotempty on non-collection field `{}` in type `{}`",
                            field_def.name, pt.singular_name
                        ),
                        "@ifnotempty is only meaningful for collection fields (plural types or cardinalized fields)"
                            .to_string(),
                    ));
                }
            }

            // Validate body (inside guard context)
            if body.is_empty() {
                diags.push(super::error(
                    file,
                    format!(
                        "@ifnotempty block for `{}` in type `{}` has empty body",
                        field_id_name(pt, *field),
                        pt.singular_name
                    ),
                ));
            }
            for child in body {
                validate_render_node(child, pt, module, file, diags, true);
            }
        }

        RenderNode::Text(_) => {
            // String literals are always valid
        }
    }
}

/// T-064: Validate separator field is a singular scalar string.
fn validate_separator_field(
    field: &FieldDef,
    pt: &PhenotypeType,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if !is_scalar_string(&field.ty) {
        diags.push(super::error(
            file,
            format!(
                "@join separator field `{}` in type `{}` must be a string type",
                field.name, pt.singular_name
            ),
        ));
    }
    if field.cardinality != Cardinality::One {
        diags.push(super::error(
            file,
            format!(
                "@join separator field `{}` in type `{}` must be singular (not a collection)",
                field.name, pt.singular_name
            ),
        ));
    }
}

/// T-065: Detect cyclic type references.
///
/// A type graph has a cycle if type A has a required field of type A (directly
/// or transitively). Optional fields and collection fields break cycles.
fn validate_cyclic_types(module: &PhenotypeModule, file: &str, diags: &mut Vec<Diagnostic>) {
    for pt in &module.types {
        let mut visited = HashSet::new();
        if has_cycle(pt.id, module, &mut visited) {
            diags.push(super::error_with_explanation(
                file,
                format!("cyclic type definition involving `{}`", pt.singular_name),
                "recursive type definitions are not supported in v1".to_string(),
            ));
        }
    }
}

/// Check if following required, singular fields from `type_id` leads back to it.
fn has_cycle(type_id: TypeId, module: &PhenotypeModule, visited: &mut HashSet<TypeId>) -> bool {
    if !visited.insert(type_id) {
        return true;
    }

    if let Some(pt) = module.types.iter().find(|t| t.id == type_id) {
        for field in &pt.fields {
            // Only required, singular fields create true cycles
            if field.requiredness == Requiredness::Required && field.cardinality == Cardinality::One
            {
                if let Some(ref_id) = referenced_type_id(&field.ty) {
                    if has_cycle(ref_id, module, visited) {
                        return true;
                    }
                }
            }
        }
    }

    visited.remove(&type_id);
    false
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Find a field by its `FieldId` within a phenotype type.
fn find_field(pt: &PhenotypeType, id: FieldId) -> Option<&FieldDef> {
    pt.fields.iter().find(|f| f.id == id)
}

/// Get the field name for a `FieldId`, or "<unknown>".
fn field_id_name(pt: &PhenotypeType, id: FieldId) -> String {
    find_field(pt, id)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string())
}

/// Check if a value type is a scalar string primitive.
fn is_scalar_string(ty: &ValueType) -> bool {
    matches!(
        ty,
        ValueType::Primitive(crate::symbol::PrimitiveType::String)
    )
}

/// Check if a field represents a collection (plural type or cardinalized).
fn is_collection_type(ty: &ValueType, cardinality: &Cardinality) -> bool {
    // Cardinality-based collections
    if *cardinality != Cardinality::One {
        return true;
    }
    // Plural type references are collections — local or imported
    matches!(
        ty,
        ValueType::UserPlural { .. }
            | ValueType::Imported(crate::ir::ImportedRef {
                kind: crate::symbol::ImportedKind::Plural,
                ..
            })
    )
}

/// Extract the referenced TypeId from a value type, if it references a phenotype.
fn referenced_type_id(ty: &ValueType) -> Option<TypeId> {
    match ty {
        ValueType::UserSingular(id) => Some(*id),
        ValueType::UserPlural { collection_of } => Some(*collection_of),
        _ => None,
    }
}
