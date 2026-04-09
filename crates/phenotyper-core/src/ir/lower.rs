// SPDX-License-Identifier: Apache-2.0
//! AST-to-IR lowering pass (REQ-COMP-007).
//!
//! Walks the AST + symbol table and produces the normalized IR:
//! - Directive canonicalization: `@(field)` → `Emit(field_id)`
//! - Eol lowering: `@eol` → `Eol { field: None }`, `@eol(f)` → `Eol { field: Some(id) }`
//! - Union flattening: nested unions recursively flattened
//! - Primitive recognition: `string` etc. → `PrimitiveType`
//! - Singular/plural normalization: plural ref → `UserPlural { collection_of }`
//! - Cardinality normalization: `+` → `OneOrMore`, `*` → `ZeroOrMore`
//! - Block directive lowering: `@ifset`/`@ifnotempty` → `IfSet`/`IfNotEmpty`

use crate::diagnostic::Diagnostic;
use crate::parser::phenotyper_actions as ast;
use crate::symbol::{
    Cardinality, FieldId, PrimitiveType, Requiredness, Symbol, SymbolTable, TypeId,
};

use super::{
    EnumType, FieldDef, PhenotypeModule, PhenotypeType, RenderNode, SeparatorExpr, TypeAlias,
    ValueType,
};

/// Lower the entire module.
pub fn lower_module(
    file_ast: &ast::File,
    table: &SymbolTable,
    file: &str,
) -> (PhenotypeModule, Vec<Diagnostic>) {
    let mut diags = Vec::new();

    let namespace = table.namespace.join("/");

    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut aliases = Vec::new();

    if let Some(ref decls) = file_ast.ns.decls {
        for decl in decls {
            match decl {
                ast::TopLevelDecl::TypeDecl(td) => {
                    lower_type_decl(td, table, file, &mut diags, &mut enums, &mut aliases);
                }
                ast::TopLevelDecl::TypeDef(td) => {
                    if let Some(pt) = lower_type_def(td, table, file, &mut diags, &mut types) {
                        types.push(pt);
                    }
                }
            }
        }
    }

    let module = PhenotypeModule {
        namespace,
        types,
        enums,
        aliases,
    };

    (module, diags)
}

/// Lower a type declaration (alias or enum).
fn lower_type_decl(
    decl: &ast::TypeDecl,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
    enums: &mut Vec<EnumType>,
    aliases: &mut Vec<TypeAlias>,
) {
    match &decl.body {
        ast::TypeDeclBody::Alias(alias) => {
            let id = match table.resolve(&decl.name) {
                Some(Symbol::Alias(aid)) => *aid,
                _ => return,
            };
            let target = lower_type_expr(&alias.alias, table, file, diags);
            aliases.push(TypeAlias {
                id,
                name: decl.name.clone(),
                target,
            });
        }
        ast::TypeDeclBody::Enum(enum_decl) => {
            let id = match table.resolve(&decl.name) {
                Some(Symbol::Enum(eid)) => *eid,
                _ => return,
            };
            let members = flatten_enum_members(&enum_decl.members);
            enums.push(EnumType {
                id,
                name: decl.name.clone(),
                members,
            });
        }
    }
}

/// Lower a phenotype type definition.
///
/// Nested phenotype definitions (BodyItem::NestedType) are recursively lowered
/// and flattened into the `extra_types` output vec — the IR is always flat.
fn lower_type_def(
    def: &ast::TypeDef,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
    extra_types: &mut Vec<PhenotypeType>,
) -> Option<PhenotypeType> {
    let type_id = match table.resolve(&def.name) {
        Some(Symbol::Phenotype(id)) => *id,
        _ => return None,
    };

    let type_info = table.type_info(type_id)?;

    let plural_name = type_info.plural_name.clone();

    // Lower fields
    let mut fields = Vec::new();
    for item in &def.items {
        if let ast::BodyItem::Field(f) = item {
            if let Some(field_def) = lower_field_decl(&f.field, type_id, table, file, diags) {
                fields.push(field_def);
            }
        }
    }

    // Recursively lower nested phenotype definitions (flattened into extra_types)
    for item in &def.items {
        if let ast::BodyItem::NestedType(nt) | ast::BodyItem::NestedTypePlural(nt) = item {
            if let Some(nested_pt) = lower_type_def(&nt.nested, table, file, diags, extra_types) {
                extra_types.push(nested_pt);
            }
        }
    }

    // Lower render expressions (pass fields for ? desugaring)
    let mut render = Vec::new();
    for item in &def.items {
        if let ast::BodyItem::Render(r) = item {
            if let Some(node) = lower_render_expr(&r.render, type_id, &fields, table, file, diags) {
                render.push(node);
            }
        }
    }

    Some(PhenotypeType {
        id: type_id,
        singular_name: def.name.clone(),
        plural_name,
        fields,
        render,
    })
}

/// Lower a field declaration.
fn lower_field_decl(
    field: &ast::FieldDecl,
    type_id: TypeId,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<FieldDef> {
    let field_info = table.resolve_field(type_id, &field.name)?;

    let requiredness = match field.req {
        ast::Requiredness::Req => Requiredness::Required,
        ast::Requiredness::Opt => Requiredness::Optional,
    };

    let (ty, cardinality) = lower_type_expr_with_cardinality(&field.type_expr, table, file, diags);

    Some(FieldDef {
        id: field_info.id,
        name: field.name.clone(),
        requiredness,
        cardinality,
        ty,
    })
}

/// Lower a type expression, extracting cardinality.
fn lower_type_expr_with_cardinality(
    expr: &ast::TypeExpr,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> (ValueType, Cardinality) {
    match expr {
        ast::TypeExpr::Simple(simple) => {
            let vt = lower_type_name(&simple.name, table, file, diags);
            (vt, Cardinality::One)
        }
        ast::TypeExpr::Cardinalized(card) => {
            let vt = lower_type_name(&card.base, table, file, diags);
            let cardinality = match card.card {
                ast::CardinalityOp::Plus => Cardinality::OneOrMore,
                ast::CardinalityOp::Star => Cardinality::ZeroOrMore,
            };
            (vt, cardinality)
        }
        ast::TypeExpr::Union(union) => {
            let vt = lower_union_type(&union.members, table, file, diags);
            (vt, Cardinality::One)
        }
    }
}

/// Lower a type expression (without cardinality extraction).
fn lower_type_expr(
    expr: &ast::TypeExpr,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> ValueType {
    let (vt, _) = lower_type_expr_with_cardinality(expr, table, file, diags);
    vt
}

/// Lower a single type name to a `ValueType`.
fn lower_type_name(
    name: &ast::TypeName,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> ValueType {
    match name {
        // Primitives
        ast::TypeName::String => ValueType::Primitive(PrimitiveType::String),
        ast::TypeName::Int64 => ValueType::Primitive(PrimitiveType::Int64),
        ast::TypeName::Real64 => ValueType::Primitive(PrimitiveType::Real64),
        ast::TypeName::Bool => ValueType::Primitive(PrimitiveType::Bool),
        ast::TypeName::Date => ValueType::Primitive(PrimitiveType::Date),
        ast::TypeName::Time => ValueType::Primitive(PrimitiveType::Time),
        ast::TypeName::DateTime => ValueType::Primitive(PrimitiveType::DateTime),

        // User-defined: resolve via symbol table
        ast::TypeName::UserDefined(ud) => match table.resolve(&ud.name) {
            Some(Symbol::Phenotype(id)) => ValueType::UserSingular(*id),
            Some(Symbol::PluralCompanion { type_id }) => ValueType::UserPlural {
                collection_of: *type_id,
            },
            Some(Symbol::Enum(id)) => ValueType::Enum(*id),
            Some(Symbol::Alias(id)) => ValueType::TypeAlias(*id),
            None => {
                // Already reported by symbol resolution pass, but be defensive
                diags.push(super::error(
                    file,
                    format!("unresolved type `{}` during IR lowering", ud.name),
                ));
                ValueType::Primitive(PrimitiveType::String) // placeholder
            }
        },
    }
}

/// Lower a union type expression, flattening nested unions.
fn lower_union_type(
    members: &[ast::TypeName],
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> ValueType {
    let mut flat = Vec::new();
    for member in members {
        let vt = lower_type_name(member, table, file, diags);
        // Flatten nested unions
        if let ValueType::Union(inner) = vt {
            flat.extend(inner);
        } else {
            flat.push(vt);
        }
    }
    ValueType::Union(flat)
}

/// Extract the field name from a `FieldPath`.
///
/// For single-segment paths like `@(name)`, returns `"name"`.
/// For multi-segment scoped paths like `@(Parent/field)`, returns `"field"` —
/// the scope qualifier is resolved separately during symbol resolution.
fn resolve_field_path_name(
    path: &ast::FieldPath,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<String> {
    if path.segments.is_empty() {
        diags.push(super::error(file, "empty field path".to_string()));
        return None;
    }
    // Return the last segment as the field name
    Some(path.segments.last().unwrap().clone())
}

/// Lower a render expression to an IR `RenderNode`.
fn lower_render_expr(
    expr: &ast::RenderExpr,
    type_id: TypeId,
    fields: &[FieldDef],
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<RenderNode> {
    match expr {
        // @(field) or @(Parent/field) → Emit(field_id)
        ast::RenderExpr::FieldRef(fr) => {
            let field_name = resolve_field_path_name(&fr.ref_path, file, diags)?;
            let field_id = resolve_field_id(&field_name, type_id, table, file, diags)?;
            Some(RenderNode::Emit(field_id))
        }

        // "literal" → Text(string)
        ast::RenderExpr::StringLit(sl) => {
            // Strip surrounding quotes from the string literal
            let value = strip_string_quotes(&sl.value);
            Some(RenderNode::Text(value))
        }

        // @eol (bare) → Eol { field: None }
        ast::RenderExpr::BareDirective(bd) => {
            if bd.name == "eol" {
                Some(RenderNode::Eol { field: None })
            } else {
                diags.push(super::error(
                    file,
                    format!("unknown bare directive `@{}`", bd.name),
                ));
                None
            }
        }

        // @name(...) with or without block body
        ast::RenderExpr::Directive(d) => lower_named_directive(d, type_id, table, file, diags),

        // @(field)? or @(field)? { body } — desugar to IfSet/IfNotEmpty
        ast::RenderExpr::ConditionalRef(cr) => {
            lower_conditional_ref(cr, type_id, fields, table, file, diags)
        }

        // @name(...)? or @name(...)? { body } — desugar directive with ?
        ast::RenderExpr::ConditionalDirective(cd) => {
            lower_conditional_directive(cd, type_id, fields, table, file, diags)
        }
    }
}

/// Lower a named directive (`@name(args) { body? }`).
fn lower_named_directive(
    d: &ast::Directive,
    type_id: TypeId,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<RenderNode> {
    let name = d.name.as_str();
    match &d.suffix {
        ast::DirectiveSuffix::WithArgs(wa) => {
            match name {
                "eol" => {
                    // @eol(field) → Eol { field: Some(id) }
                    let arg = single_ident_arg(&wa.args, "@eol", file, diags)?;
                    let field_id = resolve_field_id(&arg, type_id, table, file, diags)?;
                    Some(RenderNode::Eol {
                        field: Some(field_id),
                    })
                }
                "join" => {
                    // @join(field, separator) → Join { field, separator }
                    if wa.args.len() != 2 {
                        diags.push(super::error(
                            file,
                            format!("@join requires exactly 2 arguments, got {}", wa.args.len()),
                        ));
                        return None;
                    }
                    let field_name = ident_arg(&wa.args[0], "@join", file, diags)?;
                    let field_id = resolve_field_id(&field_name, type_id, table, file, diags)?;
                    let separator = lower_separator_arg(&wa.args[1], type_id, table, file, diags)?;
                    Some(RenderNode::Join {
                        field: field_id,
                        separator,
                    })
                }
                "ifset" => {
                    // @ifset(field) { body } → IfSet { field, body }
                    let arg = single_ident_arg(&wa.args, "@ifset", file, diags)?;
                    let field_id = resolve_field_id(&arg, type_id, table, file, diags)?;
                    let body = lower_block_body(&wa.block, type_id, table, file, diags);
                    Some(RenderNode::IfSet {
                        field: field_id,
                        body,
                    })
                }
                "ifnotempty" => {
                    // @ifnotempty(field) { body } → IfNotEmpty { field, body }
                    let arg = single_ident_arg(&wa.args, "@ifnotempty", file, diags)?;
                    let field_id = resolve_field_id(&arg, type_id, table, file, diags)?;
                    let body = lower_block_body(&wa.block, type_id, table, file, diags);
                    Some(RenderNode::IfNotEmpty {
                        field: field_id,
                        body,
                    })
                }
                _ => {
                    diags.push(super::error(file, format!("unknown directive `@{name}`")));
                    None
                }
            }
        }
        ast::DirectiveSuffix::EmptyParen(ep) => {
            match name {
                "eol" => {
                    // @eol() → Eol { field: None }
                    Some(RenderNode::Eol { field: None })
                }
                _ => {
                    if ep.block.is_some() {
                        diags.push(super::error(
                            file,
                            format!("`@{name}()` with block body is not supported"),
                        ));
                    } else {
                        diags.push(super::error(file, format!("unknown directive `@{name}()`")));
                    }
                    None
                }
            }
        }
    }
}

/// Lower a `@(field)?` or `@(field)? { body }` conditional reference.
///
/// Desugars based on field type:
/// - `optional` field → `IfSet { field, body }`
/// - Collection field (`*`/`+`) → `IfNotEmpty { field, body }`
/// - `required` scalar → treated as `IfSet` (semantic pass warns)
fn lower_conditional_ref(
    cr: &ast::ConditionalRef,
    type_id: TypeId,
    fields: &[FieldDef],
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<RenderNode> {
    let field_name = resolve_field_path_name(&cr.ref_path, file, diags)?;
    let field_id = resolve_field_id(&field_name, type_id, table, file, diags)?;

    // Determine body: if block is present, lower it; else default to [Emit(field)]
    let body = if let Some(ref block) = cr.block {
        let mut nodes = Vec::new();
        for item in &block.items {
            if let Some(node) = lower_render_expr(item, type_id, fields, table, file, diags) {
                nodes.push(node);
            }
        }
        nodes
    } else {
        vec![RenderNode::Emit(field_id)]
    };

    // Decide IfSet vs IfNotEmpty based on cardinality
    let field_def = fields.iter().find(|f| f.id == field_id);
    let is_collection = field_def
        .map(|f| {
            matches!(
                f.cardinality,
                Cardinality::OneOrMore | Cardinality::ZeroOrMore
            )
        })
        .unwrap_or(false);

    if is_collection {
        Some(RenderNode::IfNotEmpty {
            field: field_id,
            body,
        })
    } else {
        Some(RenderNode::IfSet {
            field: field_id,
            body,
        })
    }
}

/// Lower a `@name(args)?` or `@name(args)? { body }` conditional directive.
///
/// Currently only `@join(field, sep)?` is defined:
/// - Bare: `@join(f, s)?` → `IfNotEmpty { f, [Join(f, s)] }`
/// - Block: `@join(f, s)? { body }` → `IfNotEmpty { f, body }`
fn lower_conditional_directive(
    cd: &ast::ConditionalDirective,
    type_id: TypeId,
    fields: &[FieldDef],
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<RenderNode> {
    let name = cd.name.as_str();

    match &cd.suffix {
        ast::DirectiveSuffix::WithArgs(wa) => match name {
            "join" => {
                if wa.args.len() != 2 {
                    diags.push(super::error(
                        file,
                        format!("@join requires exactly 2 arguments, got {}", wa.args.len()),
                    ));
                    return None;
                }
                let field_name = ident_arg(&wa.args[0], "@join", file, diags)?;
                let field_id = resolve_field_id(&field_name, type_id, table, file, diags)?;
                let separator = lower_separator_arg(&wa.args[1], type_id, table, file, diags)?;

                // Determine body: block or default join
                let body = if let Some(ref block) = cd.block {
                    let mut nodes = Vec::new();
                    for item in &block.items {
                        if let Some(node) =
                            lower_render_expr(item, type_id, fields, table, file, diags)
                        {
                            nodes.push(node);
                        }
                    }
                    nodes
                } else {
                    vec![RenderNode::Join {
                        field: field_id,
                        separator,
                    }]
                };

                Some(RenderNode::IfNotEmpty {
                    field: field_id,
                    body,
                })
            }
            _ => {
                diags.push(super::error(
                    file,
                    format!("`?` suffix is not supported on `@{name}`"),
                ));
                None
            }
        },
        ast::DirectiveSuffix::EmptyParen(_) => {
            diags.push(super::error(
                file,
                format!("`@{name}()?` is not a valid conditional directive"),
            ));
            None
        }
    }
}

/// Lower the body of a block directive.
fn lower_block_body(
    block: &Option<ast::BlockBody>,
    type_id: TypeId,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    if let Some(body) = block {
        for item in &body.items {
            // Block bodies inside @ifset/@ifnotempty don't need fields for ? desugaring
            // (nested ? is allowed but rare — pass empty slice)
            if let Some(node) = lower_render_expr(item, type_id, &[], table, file, diags) {
                nodes.push(node);
            }
        }
    }
    nodes
}

/// Lower a separator argument (can be a string literal or field ident).
fn lower_separator_arg(
    arg: &ast::Argument,
    type_id: TypeId,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<SeparatorExpr> {
    match arg {
        ast::Argument::LiteralArg(lit) => {
            Some(SeparatorExpr::Literal(strip_string_quotes(&lit.val)))
        }
        ast::Argument::IdentArg(id) => {
            let field_id = resolve_field_id(&id.val, type_id, table, file, diags)?;
            Some(SeparatorExpr::Field(field_id))
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Extract a single identifier argument from an argument list.
fn single_ident_arg(
    args: &[ast::Argument],
    directive: &str,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<String> {
    if args.len() != 1 {
        diags.push(super::error(
            file,
            format!(
                "{directive} requires exactly 1 argument, got {}",
                args.len()
            ),
        ));
        return None;
    }
    ident_arg(&args[0], directive, file, diags)
}

/// Extract an identifier from an argument.
fn ident_arg(
    arg: &ast::Argument,
    directive: &str,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<String> {
    match arg {
        ast::Argument::IdentArg(id) => Some(id.val.clone()),
        ast::Argument::LiteralArg(_) => {
            diags.push(super::error(
                file,
                format!("{directive} expects an identifier argument, not a string literal"),
            ));
            None
        }
    }
}

/// Resolve a field name to its `FieldId` within a type.
fn resolve_field_id(
    field_name: &str,
    type_id: TypeId,
    table: &SymbolTable,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<FieldId> {
    match table.resolve_field(type_id, field_name) {
        Some(fi) => Some(fi.id),
        None => {
            diags.push(super::error(
                file,
                format!("unknown field `{field_name}` during IR lowering"),
            ));
            None
        }
    }
}

/// Strip surrounding double quotes from a string literal token value.
fn strip_string_quotes(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        // Also process escape sequences
        let inner = &s[1..s.len() - 1];
        unescape(inner)
    } else {
        s.to_string()
    }
}

/// Basic escape sequence processing for string literals.
fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Flatten the recursive `EnumMembers` AST to a flat `Vec<String>`.
fn flatten_enum_members(members: &ast::EnumMembers) -> Vec<String> {
    let mut result = Vec::new();
    flatten_inner(members, &mut result);
    result
}

fn flatten_inner(members: &ast::EnumMembers, out: &mut Vec<String>) {
    match members {
        ast::EnumMembers::Cons(cons) => {
            out.push(cons.first.clone());
            flatten_inner(&cons.rest, out);
        }
        ast::EnumMembers::Single(single) => {
            out.push(single.last.clone());
        }
    }
}
