// SPDX-License-Identifier: Apache-2.0
//! Pass 1: Symbol collection — walk the AST and register all declarations.

use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::parser::phenotyper_actions as ast;

use super::{
    AliasId, AliasInfo, EnumId, EnumInfo, FieldId, FieldInfo, Requiredness, Symbol, SymbolTable,
    TypeId, TypeInfo,
};

/// Walk the AST and build the initial symbol table with all declarations.
///
/// Detects:
/// - Duplicate type names (singular, plural, enum, alias)
/// - Singular/plural name collisions
/// - Duplicate field names within a type
/// - Duplicate enum member names
pub fn collect_symbols(
    file_ast: &ast::File,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> SymbolTable {
    let mut table = SymbolTable {
        namespace: file_ast.ns.path.clone(),
        types: Vec::new(),
        enums: Vec::new(),
        aliases: Vec::new(),
        names: HashMap::new(),
    };

    // Process top-level declarations
    if let Some(ref decls) = file_ast.ns.decls {
        for decl in decls {
            match decl {
                ast::TopLevelDecl::TypeDecl(td) => {
                    collect_type_decl(td, file, diags, &mut table);
                }
                ast::TopLevelDecl::TypeDef(td) => {
                    collect_type_def(td, file, diags, &mut table);
                }
            }
        }
    }

    table
}

/// Collect a `type Name: ...;` declaration (alias or enum).
fn collect_type_decl(
    decl: &ast::TypeDecl,
    file: &str,
    diags: &mut Vec<Diagnostic>,
    table: &mut SymbolTable,
) {
    let name = &decl.name;

    match &decl.body {
        ast::TypeDeclBody::Alias(_alias) => {
            let alias_id = AliasId(table.aliases.len());

            if table.names.contains_key(name) {
                diags.push(super::error(file, format!("duplicate type name `{name}`")));
                return;
            }

            table.aliases.push(AliasInfo {
                id: alias_id,
                name: name.clone(),
            });
            table.names.insert(name.clone(), Symbol::Alias(alias_id));
        }
        ast::TypeDeclBody::Enum(enum_decl) => {
            let enum_id = EnumId(table.enums.len());

            if table.names.contains_key(name) {
                diags.push(super::error(file, format!("duplicate type name `{name}`")));
                return;
            }

            // Collect enum members and check for duplicates
            let members = flatten_enum_members(&enum_decl.members);
            let mut seen_members = HashMap::new();
            for member in &members {
                if let Some(_prev) = seen_members.insert(member.clone(), ()) {
                    diags.push(super::error(
                        file,
                        format!("duplicate enum member `{member}` in enum `{name}`"),
                    ));
                }
            }

            table.enums.push(EnumInfo {
                id: enum_id,
                name: name.clone(),
                members,
            });
            table.names.insert(name.clone(), Symbol::Enum(enum_id));
        }
    }
}

/// Collect a phenotype (type definition) declaration.
fn collect_type_def(
    def: &ast::TypeDef,
    file: &str,
    diags: &mut Vec<Diagnostic>,
    table: &mut SymbolTable,
) {
    let singular = &def.name;
    let type_id = TypeId(table.types.len());

    // Check singular name collision
    if table.names.contains_key(singular) {
        diags.push(super::error(
            file,
            format!("duplicate type name `{singular}`"),
        ));
        return;
    }

    // Extract plural name
    let plural = def.plural_clause.as_ref().map(|pc| pc.plural_name.clone());

    // Check plural name collision
    if let Some(ref plural_name) = plural {
        if table.names.contains_key(plural_name) {
            diags.push(super::error(
                file,
                format!("duplicate type name `{plural_name}` (plural companion of `{singular}`)"),
            ));
            return;
        }
        // Singular/plural collision (same name)
        if plural_name == singular {
            diags.push(super::error(
                file,
                format!(
                    "singular name `{singular}` and plural name `{plural_name}` must be different"
                ),
            ));
            return;
        }
    }

    // Collect fields and check for duplicates
    let mut fields = Vec::new();
    let mut field_names: HashMap<String, ()> = HashMap::new();
    let mut field_id_counter = 0usize;

    for item in &def.items {
        if let ast::BodyItem::Field(f) = item {
            let field_name = &f.field.name;

            if field_names.contains_key(field_name) {
                diags.push(super::error(
                    file,
                    format!("duplicate field `{field_name}` in type `{singular}`"),
                ));
                continue;
            }

            let req = match f.field.req {
                ast::Requiredness::Req => Requiredness::Required,
                ast::Requiredness::Opt => Requiredness::Optional,
            };

            fields.push(FieldInfo {
                id: FieldId(field_id_counter),
                name: field_name.clone(),
                requiredness: req,
            });
            field_names.insert(field_name.clone(), ());
            field_id_counter += 1;
        }
    }

    // Register the type
    table.types.push(TypeInfo {
        id: type_id,
        singular_name: singular.clone(),
        plural_name: plural.clone(),
        fields,
    });
    table
        .names
        .insert(singular.clone(), Symbol::Phenotype(type_id));

    // Register the plural companion name
    if let Some(plural_name) = plural {
        table
            .names
            .insert(plural_name, Symbol::PluralCompanion { type_id });
    }

    // Recursively collect nested phenotype definitions
    for item in &def.items {
        if let ast::BodyItem::NestedType(nt) | ast::BodyItem::NestedTypePlural(nt) = item {
            collect_type_def(&nt.nested, file, diags, table);
        }
    }
}

/// Flatten the recursive `EnumMembers` structure to a flat `Vec<String>`.
fn flatten_enum_members(members: &ast::EnumMembers) -> Vec<String> {
    let mut result = Vec::new();
    flatten_enum_members_inner(members, &mut result);
    result
}

fn flatten_enum_members_inner(members: &ast::EnumMembers, out: &mut Vec<String>) {
    match members {
        ast::EnumMembers::Cons(cons) => {
            out.push(cons.first.clone());
            flatten_enum_members_inner(&cons.rest, out);
        }
        ast::EnumMembers::Single(single) => {
            out.push(single.last.clone());
        }
    }
}
