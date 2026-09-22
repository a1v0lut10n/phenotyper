// SPDX-License-Identifier: Apache-2.0
//! Symbol table module — name collection and resolution.
//!
//! Implements a two-pass analysis of the Rustemo-generated AST:
//!
//! 1. **Collection** — Walk all declarations, register names, detect duplicates.
//! 2. **Resolution** — Resolve type references in fields and field references in
//!    render expressions; emit errors for unknown names.
//!
//! # Usage
//!
//! ```ignore
//! use phenotyper::symbol;
//! let (table, diagnostics) = symbol::build(&ast, "file.pht");
//! ```

mod collect;
mod resolve;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::phenotyper_actions as ast;

// ─── Identifiers ────────────────────────────────────────────────────────────

/// Unique identifier for a phenotype (type definition).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

/// Unique identifier for an enum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(pub usize);

/// Unique identifier for a type alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AliasId(pub usize);

/// Unique identifier for a field within a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub usize);

// ─── Primitives ─────────────────────────────────────────────────────────────

/// Built-in primitive types (lexically reserved per DEC-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    Int64,
    Real64,
    Bool,
    Date,
    Time,
    DateTime,
}

// ─── Symbols ────────────────────────────────────────────────────────────────

/// What a name in the global scope resolves to.
#[derive(Debug, Clone)]
pub enum Symbol {
    /// A phenotype type (the singular name).
    Phenotype(TypeId),
    /// A plural companion name that maps back to a phenotype.
    PluralCompanion { type_id: TypeId },
    /// An enum type.
    Enum(EnumId),
    /// A type alias.
    Alias(AliasId),
}

// ─── Imports (`uses`, REQ-LANG-003) ─────────────────────────────────────────

/// What kind of declaration an imported name is, from the exporter's view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedKind {
    /// A phenotype (singular name).
    Phenotype,
    /// A plural companion collection type.
    Plural,
    /// An enum type.
    Enum,
    /// A type alias.
    Alias,
}

/// A declaration brought into unqualified scope by a `uses` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSymbol {
    /// The namespace the declaration lives in (e.g. `"aivolution/core/time"`).
    pub namespace: String,
    /// The declared name.
    pub name: String,
    pub kind: ImportedKind,
}

/// How an imported name resolves in this file's scope.
#[derive(Debug, Clone)]
pub enum ImportResolution {
    /// Exactly one `uses` namespace exports the name.
    One(ImportedSymbol),
    /// More than one does — a hard error at any unqualified use site
    /// (REQ-LANG-003); the vec lists the exporting namespaces.
    Ambiguous(Vec<String>),
}

/// The names a compiled namespace offers to importers — everything an
/// importing file needs for resolution and codegen, and nothing more
/// (imported declarations are referenced by Rust path, never re-emitted).
#[derive(Debug, Clone, Default)]
pub struct NamespaceExports {
    /// The exporting namespace path (e.g. `"aivolution/core/time"`).
    pub namespace: String,
    /// Exported name → kind.
    pub names: HashMap<String, ImportedKind>,
}

/// A name looked up across local declarations and imports. Locals win
/// (shadowing is warned about at collection time).
#[derive(Debug, Clone)]
pub enum ResolvedSymbol<'a> {
    Local(&'a Symbol),
    Imported(&'a ImportedSymbol),
    /// The name is importable from several namespaces and not declared
    /// locally: using it is an error naming the candidates.
    Ambiguous(&'a [String]),
}

// ─── Type info ──────────────────────────────────────────────────────────────

/// Requiredness of a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requiredness {
    Required,
    Optional,
}

/// Cardinality modifier on a type expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    One,
    OneOrMore,
    ZeroOrMore,
}

/// Information about a declared phenotype.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub id: TypeId,
    pub singular_name: String,
    pub plural_name: Option<String>,
    pub fields: Vec<FieldInfo>,
}

/// Information about a declared field.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub id: FieldId,
    pub name: String,
    pub requiredness: Requiredness,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub id: EnumId,
    pub name: String,
    pub members: Vec<String>,
}

/// Information about a type alias.
#[derive(Debug, Clone)]
pub struct AliasInfo {
    pub id: AliasId,
    pub name: String,
}

// ─── Symbol table ───────────────────────────────────────────────────────────

/// The symbol table — output of collection and resolution passes.
#[derive(Debug)]
pub struct SymbolTable {
    /// Namespace path segments (e.g., `["aivolution", "format", "csv"]`).
    pub namespace: Vec<String>,
    /// All declared phenotype types, indexed by `TypeId`.
    pub types: Vec<TypeInfo>,
    /// All declared enum types, indexed by `EnumId`.
    pub enums: Vec<EnumInfo>,
    /// All declared type aliases, indexed by `AliasId`.
    pub aliases: Vec<AliasInfo>,
    /// Global name registry: name → symbol.
    names: HashMap<String, Symbol>,
    /// Names imported via `uses`, populated before collection so local
    /// declarations can warn when they shadow one.
    imports: HashMap<String, ImportResolution>,
}

impl SymbolTable {
    /// Look up a name in the symbol table (local declarations only).
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.names.get(name)
    }

    /// Look up a name across local declarations and imports. Local names
    /// shadow imported ones (REQ-LANG-003).
    pub fn resolve_any(&self, name: &str) -> Option<ResolvedSymbol<'_>> {
        if let Some(sym) = self.names.get(name) {
            return Some(ResolvedSymbol::Local(sym));
        }
        match self.imports.get(name) {
            Some(ImportResolution::One(is)) => Some(ResolvedSymbol::Imported(is)),
            Some(ImportResolution::Ambiguous(nss)) => Some(ResolvedSymbol::Ambiguous(nss)),
            None => None,
        }
    }

    /// Whether `name` is imported (used for the shadow warning).
    fn imported(&self, name: &str) -> Option<&ImportResolution> {
        self.imports.get(name)
    }

    /// The names this file offers to importers.
    pub fn exports(&self) -> NamespaceExports {
        let mut names = HashMap::new();
        for (name, sym) in &self.names {
            let kind = match sym {
                Symbol::Phenotype(_) => ImportedKind::Phenotype,
                Symbol::PluralCompanion { .. } => ImportedKind::Plural,
                Symbol::Enum(_) => ImportedKind::Enum,
                Symbol::Alias(_) => ImportedKind::Alias,
            };
            names.insert(name.clone(), kind);
        }
        NamespaceExports {
            namespace: self.namespace.join("/"),
            names,
        }
    }

    /// Look up a field by name within a specific type.
    pub fn resolve_field(&self, type_id: TypeId, field_name: &str) -> Option<&FieldInfo> {
        self.types
            .get(type_id.0)
            .and_then(|ti| ti.fields.iter().find(|f| f.name == field_name))
    }

    /// Get the type info for a `TypeId`.
    pub fn type_info(&self, id: TypeId) -> Option<&TypeInfo> {
        self.types.get(id.0)
    }

    /// Get the enum info for an `EnumId`.
    pub fn enum_info(&self, id: EnumId) -> Option<&EnumInfo> {
        self.enums.get(id.0)
    }
}

// ─── Entry point ────────────────────────────────────────────────────────────

/// Build the symbol table from a parsed AST (no imports in scope).
///
/// Returns the symbol table and a list of diagnostics (errors/warnings).
/// The table is always returned (even if incomplete) so that downstream
/// passes can attempt partial analysis. A file whose `uses` declarations
/// cannot be satisfied (none are, here) gets one error per declaration —
/// compile through a compilation set (`compile_with_roots`) to satisfy them.
pub fn build(ast: &ast::File, file: &str) -> (SymbolTable, Vec<Diagnostic>) {
    build_with_imports(ast, file, &[])
}

/// Build the symbol table with the exports of the namespaces this file
/// `uses` in scope (REQ-LANG-003).
pub fn build_with_imports(
    ast: &ast::File,
    file: &str,
    available: &[NamespaceExports],
) -> (SymbolTable, Vec<Diagnostic>) {
    let mut diags = Vec::new();

    let mut table = SymbolTable {
        namespace: ast.ns.path.clone(),
        types: Vec::new(),
        enums: Vec::new(),
        aliases: Vec::new(),
        names: HashMap::new(),
        imports: HashMap::new(),
    };

    // Pass 0: bring `uses` imports into scope, before collection, so local
    // declarations can warn when they shadow one.
    populate_imports(ast, &mut table, file, available, &mut diags);

    // Pass 1: Collection
    collect::collect_symbols(ast, &mut table, file, &mut diags);

    // Pass 2: Resolution
    resolve::resolve_references(ast, &mut table, file, &mut diags);

    (table, diags)
}

/// Merge the exports of every `uses` namespace into the table's import
/// scope. A name exported by more than one used namespace becomes
/// [`ImportResolution::Ambiguous`]; the error is emitted at the use site.
fn populate_imports(
    ast: &ast::File,
    table: &mut SymbolTable,
    file: &str,
    available: &[NamespaceExports],
    diags: &mut Vec<Diagnostic>,
) {
    let Some(ref uses) = ast.ns.uses else {
        return;
    };
    let own_ns = table.namespace.join("/");
    let mut seen: Vec<String> = Vec::new();

    for decl in uses {
        let ns = decl.path.join("/");
        if ns == own_ns {
            diags.push(warning(
                file,
                format!("`uses {ns};` names this file's own namespace and has no effect"),
            ));
            continue;
        }
        if seen.contains(&ns) {
            diags.push(warning(file, format!("duplicate `uses {ns};`")));
            continue;
        }
        seen.push(ns.clone());

        let Some(exports) = available.iter().find(|e| e.namespace == ns) else {
            diags.push(error(
                file,
                format!(
                    "cannot resolve `uses {ns};` — the namespace is not loaded \
                     (compile with search roots that contain `{ns}.pht` or `{ns}.md`)"
                ),
            ));
            continue;
        };

        for (name, kind) in &exports.names {
            match table.imports.get_mut(name) {
                None => {
                    table.imports.insert(
                        name.clone(),
                        ImportResolution::One(ImportedSymbol {
                            namespace: ns.clone(),
                            name: name.clone(),
                            kind: *kind,
                        }),
                    );
                }
                Some(ImportResolution::One(existing)) => {
                    let first_ns = existing.namespace.clone();
                    table.imports.insert(
                        name.clone(),
                        ImportResolution::Ambiguous(vec![first_ns, ns.clone()]),
                    );
                }
                Some(ImportResolution::Ambiguous(nss)) => {
                    nss.push(ns.clone());
                }
            }
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Create an error diagnostic.
fn error(file: &str, summary: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: None,
        suggestion: None,
    }
}

/// Create a warning diagnostic.
fn warning(file: &str, summary: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        summary,
        file: file.to_string(),
        line: 0,
        col: 0,
        explanation: None,
        suggestion: None,
    }
}
