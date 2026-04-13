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
}

impl SymbolTable {
    /// Look up a name in the symbol table.
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.names.get(name)
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

/// Build the symbol table from a parsed AST.
///
/// Returns the symbol table and a list of diagnostics (errors/warnings).
/// The table is always returned (even if incomplete) so that downstream
/// passes can attempt partial analysis.
pub fn build(ast: &ast::File, file: &str) -> (SymbolTable, Vec<Diagnostic>) {
    let mut diags = Vec::new();

    // Pass 1: Collection
    let mut table = collect::collect_symbols(ast, file, &mut diags);

    // Pass 2: Resolution
    resolve::resolve_references(ast, &mut table, file, &mut diags);

    (table, diags)
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
#[allow(dead_code)]
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
