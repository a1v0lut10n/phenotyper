// SPDX-License-Identifier: Apache-2.0
//! IR module — normalized intermediate representation.
//!
//! Lowers the Rustemo-generated AST + symbol table into a clean, desugared IR
//! ready for semantic validation and code generation.
//!
//! # Usage
//!
//! ```ignore
//! use phenotyper_core::{parser, symbol, ir};
//! let ast = parser::parse_pht(source, "file.pht")?;
//! let (table, sym_diags) = symbol::build(&ast, "file.pht");
//! let (module, ir_diags) = ir::lower(&ast, &table, "file.pht");
//! ```

mod lower;

#[cfg(test)]
mod tests;

use crate::diagnostic::{Diagnostic, Severity};
use crate::symbol::{AliasId, Cardinality, EnumId, FieldId, PrimitiveType, Requiredness, TypeId};

// ─── IR data types (REQ-COMP-006) ──────────────────────────────────────────

/// Top-level IR node representing a compiled Phenotyper file.
#[derive(Debug, Clone)]
pub struct PhenotypeModule {
    /// Fully qualified namespace (e.g., `"aivolution/format/csv"`).
    pub namespace: String,
    /// All phenotype type definitions.
    pub types: Vec<PhenotypeType>,
    /// All enum type definitions.
    pub enums: Vec<EnumType>,
    /// All type alias definitions.
    pub aliases: Vec<TypeAlias>,
}

/// A lowered phenotype type definition.
#[derive(Debug, Clone)]
pub struct PhenotypeType {
    pub id: TypeId,
    pub singular_name: String,
    pub plural_name: Option<String>,
    pub fields: Vec<FieldDef>,
    pub render: Vec<RenderNode>,
    /// If this is a nested phenotype that references parent fields,
    /// this holds the parent type name for `render_with_parent` codegen.
    pub parent_context: Option<String>,
}

/// A lowered field definition with resolved type and cardinality.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub id: FieldId,
    pub name: String,
    pub requiredness: Requiredness,
    pub cardinality: Cardinality,
    pub ty: ValueType,
}

/// A resolved value type — all names have been resolved to IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// A built-in primitive type.
    Primitive(PrimitiveType),
    /// A reference to a user-defined phenotype (singular).
    UserSingular(TypeId),
    /// A reference to a plural companion (collection of the singular type).
    UserPlural { collection_of: TypeId },
    /// A reference to an enum type.
    Enum(EnumId),
    /// A reference to a type alias.
    TypeAlias(AliasId),
    /// A union of multiple types.
    Union(Vec<ValueType>),
}

/// A lowered render expression node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderNode {
    /// Literal text: `"hello"`.
    Text(String),
    /// Emit a field value: `@(field)`.
    Emit(FieldId),
    /// Emit a parent-scoped field: `@(Parent/field)`.
    ParentFieldRef {
        /// The parent type name (e.g., `"JavaClass"`).
        parent_type: String,
        /// The field name in the parent (e.g., `"name"`).
        field_name: String,
    },
    /// Join a collection with a separator: `@join(field, separator)`.
    Join {
        field: FieldId,
        separator: SeparatorExpr,
    },
    /// End of line: `@eol` or `@eol(field)`.
    Eol { field: Option<FieldId> },
    /// Conditional on optional field being set: `@ifset(field) { body }`.
    IfSet {
        field: FieldId,
        body: Vec<RenderNode>,
    },
    /// Conditional on collection being non-empty: `@ifnotempty(field) { body }`.
    IfNotEmpty {
        field: FieldId,
        body: Vec<RenderNode>,
    },
}

/// Separator expression in a `@join` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeparatorExpr {
    /// A literal string separator.
    Literal(String),
    /// A field reference as separator.
    Field(FieldId),
}

/// A lowered enum type.
#[derive(Debug, Clone)]
pub struct EnumType {
    pub id: EnumId,
    pub name: String,
    pub members: Vec<String>,
}

/// A lowered type alias.
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub id: AliasId,
    pub name: String,
    pub target: ValueType,
}

// ─── Entry point ────────────────────────────────────────────────────────────

/// Lower the AST + symbol table into the IR.
///
/// Returns the `PhenotypeModule` and any diagnostics from the lowering pass.
pub fn lower(
    ast: &crate::parser::phenotyper_actions::File,
    table: &crate::symbol::SymbolTable,
    file: &str,
) -> (PhenotypeModule, Vec<Diagnostic>) {
    lower::lower_module(ast, table, file)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

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
