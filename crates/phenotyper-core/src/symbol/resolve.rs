// SPDX-License-Identifier: Apache-2.0
//! Pass 2: Reference resolution — resolve type and field references in the AST.

use crate::diagnostic::Diagnostic;
use crate::parser::phenotyper_actions as ast;

use super::{PrimitiveType, Symbol, SymbolTable, TypeId};

/// Resolve all type and field references in the AST against the symbol table.
///
/// Detects:
/// - Unknown type references in field type expressions
/// - Unknown field references in render expressions (`@(field)`, directive args)
pub fn resolve_references(
    file_ast: &ast::File,
    table: &mut SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if let Some(ref decls) = file_ast.ns.decls {
        for decl in decls {
            if let ast::TopLevelDecl::TypeDef(td) = decl {
                resolve_type_def(td, table, file, diags);
            }
            if let ast::TopLevelDecl::TypeDecl(td) = decl {
                resolve_type_decl(td, table, file, diags);
            }
        }
    }
}

/// Resolve references in a type declaration (alias target type).
fn resolve_type_decl(
    decl: &ast::TypeDecl,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if let ast::TypeDeclBody::Alias(alias) = &decl.body {
        resolve_type_expr(&alias.alias, &decl.name, table, file, diags);
    }
}

/// Resolve references within a phenotype definition.
fn resolve_type_def(
    def: &ast::TypeDef,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let type_name = &def.name;

    // Find the TypeId for this definition (to resolve field refs)
    let type_id = match table.resolve(type_name) {
        Some(Symbol::Phenotype(id)) => Some(*id),
        _ => None,
    };

    for item in &def.items {
        match item {
            ast::BodyItem::Field(f) => {
                resolve_type_expr(&f.field.type_expr, type_name, table, file, diags);
            }
            ast::BodyItem::Render(r) => {
                resolve_render_expr(&r.render, type_name, type_id, table, file, diags);
            }
            ast::BodyItem::NestedType(nt) | ast::BodyItem::NestedTypePlural(nt) => {
                // Recursively resolve references in nested phenotype
                resolve_type_def(&nt.nested, table, file, diags);
            }
        }
    }
}

/// Resolve a type expression (check that all user-defined type names exist).
fn resolve_type_expr(
    expr: &ast::TypeExpr,
    context_type: &str,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::TypeExpr::Simple(simple) => {
            resolve_type_name(&simple.name, context_type, table, file, diags);
        }
        ast::TypeExpr::Cardinalized(card) => {
            resolve_type_name(&card.base, context_type, table, file, diags);
        }
        ast::TypeExpr::Union(union) => {
            for member in &union.members {
                resolve_type_name(member, context_type, table, file, diags);
            }
        }
    }
}

/// Resolve a single type name — primitives always resolve, user-defined names
/// must exist in the symbol table.
fn resolve_type_name(
    name: &ast::TypeName,
    context_type: &str,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match name {
        // Primitives always resolve successfully
        ast::TypeName::String
        | ast::TypeName::Int64
        | ast::TypeName::Real64
        | ast::TypeName::Bool
        | ast::TypeName::Date
        | ast::TypeName::Time
        | ast::TypeName::DateTime => {}

        ast::TypeName::UserDefined(ud) => {
            if table.resolve(&ud.name).is_none() {
                diags.push(super::error(
                    file,
                    format!("unknown type `{}` referenced in `{context_type}`", ud.name),
                ));
            }
        }
    }
}

/// Resolve references in a render expression.
fn resolve_render_expr(
    expr: &ast::RenderExpr,
    type_name: &str,
    type_id: Option<TypeId>,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::RenderExpr::FieldRef(fr) => {
            // Multi-segment paths like @(Parent/field) reference parent fields;
            // they're validated during IR lowering, not here.
            if fr.ref_path.segments.len() == 1 {
                if let Some(field_name) = fr.ref_path.segments.last() {
                    check_field_ref(field_name, type_name, type_id, table, file, diags);
                }
            }
        }
        ast::RenderExpr::Directive(d) => {
            // Resolve field references in directive arguments
            resolve_directive_suffix(&d.suffix, type_name, type_id, table, file, diags);
        }
        ast::RenderExpr::BareDirective(_) => {
            // Bare directives like @eol have no references to resolve
        }
        ast::RenderExpr::StringLit(_) => {
            // String literals have no references
        }
        ast::RenderExpr::ConditionalRef(cr) => {
            // Multi-segment paths skip check (parent-scoped)
            if cr.ref_path.segments.len() == 1 {
                if let Some(field_name) = cr.ref_path.segments.last() {
                    check_field_ref(field_name, type_name, type_id, table, file, diags);
                }
            }
            // Resolve refs in optional block body
            if let Some(ref block) = cr.block {
                for item in &block.items {
                    resolve_render_expr(item, type_name, type_id, table, file, diags);
                }
            }
        }
        ast::RenderExpr::ConditionalDirective(cd) => {
            // Resolve field references in directive arguments
            resolve_directive_suffix(&cd.suffix, type_name, type_id, table, file, diags);
            // Resolve refs in optional block body
            if let Some(ref block) = cd.block {
                for item in &block.items {
                    resolve_render_expr(item, type_name, type_id, table, file, diags);
                }
            }
        }
    }
}

/// Resolve references in a directive suffix (arguments and block body).
fn resolve_directive_suffix(
    suffix: &ast::DirectiveSuffix,
    type_name: &str,
    type_id: Option<TypeId>,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match suffix {
        ast::DirectiveSuffix::WithArgs(wa) => {
            for arg in &wa.args {
                if let ast::Argument::IdentArg(id_arg) = arg {
                    check_field_ref(&id_arg.val, type_name, type_id, table, file, diags);
                }
            }
            if let Some(ref block) = wa.block {
                for item in &block.items {
                    resolve_render_expr(item, type_name, type_id, table, file, diags);
                }
            }
        }
        ast::DirectiveSuffix::EmptyParen(ep) => {
            if let Some(ref block) = ep.block {
                for item in &block.items {
                    resolve_render_expr(item, type_name, type_id, table, file, diags);
                }
            }
        }
    }
}

/// Check that a field reference resolves to a field in the current type.
fn check_field_ref(
    field_name: &str,
    type_name: &str,
    type_id: Option<TypeId>,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if let Some(tid) = type_id {
        if table.resolve_field(tid, field_name).is_none() {
            diags.push(super::error(
                file,
                format!("unknown field `{field_name}` in type `{type_name}`"),
            ));
        }
    }
}

/// Convert a Rustemo AST `TypeName` to our `PrimitiveType`, if it is one.
#[allow(dead_code)]
pub fn as_primitive(name: &ast::TypeName) -> Option<PrimitiveType> {
    match name {
        ast::TypeName::String => Some(PrimitiveType::String),
        ast::TypeName::Int64 => Some(PrimitiveType::Int64),
        ast::TypeName::Real64 => Some(PrimitiveType::Real64),
        ast::TypeName::Bool => Some(PrimitiveType::Bool),
        ast::TypeName::Date => Some(PrimitiveType::Date),
        ast::TypeName::Time => Some(PrimitiveType::Time),
        ast::TypeName::DateTime => Some(PrimitiveType::DateTime),
        ast::TypeName::UserDefined(_) => None,
    }
}
