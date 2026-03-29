# Phenotyper v1 Language and Compiler Specification

## Status
Draft v1 specification for language design and compiler implementation.

## 1. Purpose

This document specifies the first version of the Phenotyper language and compiler. It is intended to be concrete enough to guide implementation, while still leaving a small number of explicitly named extension points for later iterations.

Phenotyper v1 is deliberately narrow. It is a declarative language for defining structured textual artifacts and generating Rust tooling that can:

- construct conformant instances,
- render them to text,
- validate structural correctness,
- and provide a stable foundation for later parser generation.

v1 is not intended to be a general-purpose template language, scripting language, or macro system.

## 2. Goals and Non-Goals

### 2.1 Goals

Phenotyper v1 should:

1. define artifact types in a human-readable DSL,
2. express required typed fields,
3. express composition and plurality,
4. support named singular and plural phenotype declarations,
5. support a minimal set of rendering directives,
6. compile definitions into a validated intermediate representation,
7. generate ergonomic Rust builder APIs,
8. generate deterministic renderers,
9. produce actionable diagnostics for invalid definitions.

### 2.2 Non-Goals

Phenotyper v1 does not aim to support:

- arbitrary control flow,
- user-defined functions inside the DSL,
- full parser generation,
- sophisticated semantic validation beyond structural rules,
- multi-language code generation beyond Rust,
- exact import of foreign template semantics.

## 3. Design Principles

### 3.1 Structure-first
The language primarily defines artifact structure. Rendering directives are declarative annotations over structure, not a vehicle for general computation.

### 3.2 Human legibility
A phenotype definition should remain understandable as a description of the artifact family, not only as compiler input.

### 3.3 Small semantic core
v1 should have few primitives and crisp semantics. Expressiveness should come from composition rather than feature count.

### 3.4 Deterministic output
Rendering must be deterministic given a validated instance.

### 3.5 Strong normalization
The compiler should lower surface syntax into a clean IR so that code generation and later parser work are built on stable internal semantics.

### 3.6 Natural-language plurality
Plurality should be visible in the language's type system, not only encoded through punctuation. A phenotype may therefore declare both its singular and plural names, while explicit cardinality operators continue to express multiplicity constraints at use sites.

## 4. Conceptual Model

A Phenotyper source file defines one or more **phenotype types**.

Each phenotype type contains:

- a singular name,
- optionally, a declared plural companion name,
- zero or more field declarations,
- zero or more local constants or literals in later versions, but not in v1,
- a render body,
- optional metadata blocks in later versions, but not in v1.

A phenotype type describes a family of renderable artifact instances.

When a type declares a plural companion, the plural name denotes the canonical collection type associated with that singular phenotype. For example:

```text
CSVLine plural CSVLines:
    ...
;
```

defines:

- `CSVLine` as the singular phenotype name,
- `CSVLines` as the collection phenotype name associated with `CSVLine`.

A concrete artifact instance is created in Rust through generated builders, validated against compiler-generated rules, and rendered into text by generated renderer code.

## 5. Surface Language

### 5.1 General Style

Phenotyper v1 uses a punctuation-driven syntax.

- `:` introduces a type body.
- `,` separates declarations or render expressions within a body.
- `;` terminates a type definition.
- `@( ... )` introduces a directive invocation.
- string literals are delimited with double quotes.
- identifiers are ASCII word-like names: `Letter ( Letter | Digit | '_' )*`.

Whitespace is not semantically significant except as a separator between tokens.

### 5.2 File Structure

A file is a sequence of type definitions:

```text
File := TypeDefinition*
```

There is no import mechanism in v1.

### 5.3 Type Definition

Canonical forms:

```text
TypeName:
    declaration,
    declaration,
    render expression
;
```

or:

```text
SingularTypeName plural PluralTypeName:
    declaration,
    declaration,
    render expression
;
```

Illustrative example:

```text
CSVFieldValue plural CSVFieldValues:
    value: required {int64, real64, string, date, time, datetime},
    @(value)
;

CSVLine plural CSVLines:
    fields: required CSVFieldValues+,
    separator: required string,
    @join(fields, separator)
;

CSVFile:
    header: required CSVLine,
    lines: required CSVLines+,
    eol: required string,
    @(header), @eol(eol), @join(lines, eol)
;
```

This example is illustrative rather than normative. The normative semantics are defined below.

## 6. Lexical Elements

### 6.1 Identifiers

Identifiers name types and fields.

Rules:

- must begin with `[A-Za-z_]`,
- may continue with `[A-Za-z0-9_]`,
- are case-sensitive,
- type names should conventionally use `PascalCase`,
- field names should conventionally use `snake_case` or `lowerCamelCase`; v1 compiler accepts either.

### 6.2 Keywords

Reserved words in v1:

- `required`
- `optional`
- `plural`

Future versions may reserve additional words.

### 6.3 Literals

v1 supports:

- string literals: `"text"`

Numeric literals are not needed in the language itself in v1 because numbers appear as runtime field values, not compile-time syntax.

### 6.4 Comments

Recommended v1 comment forms:

- line comment: `// ...`
- block comment: `/* ... */`

Comments are ignored by the parser.

## 7. Type System

### 7.1 Primitive Types

Phenotyper v1 primitive scalar types:

- `string`
- `int64`
- `real64`
- `bool`
- `date`
- `time`
- `datetime`

These are semantic scalar categories. Rust code generation decides their concrete backing types.

### 7.2 Type References

A field may reference:

- a primitive type,
- a user-defined singular phenotype type,
- a user-defined plural phenotype type,
- a union of primitive types or user-defined singular types,
- a cardinality-annotated form of one of the above where permitted.

### 7.3 Singular and Plural Phenotype Names

A type definition may declare both a singular and a plural name:

```text
CSVLine plural CSVLines:
    ...
;
```

Semantics:

- the singular name (`CSVLine`) denotes one instance of the phenotype,
- the plural name (`CSVLines`) denotes the collection phenotype associated with that singular type,
- the plural companion is not merely cosmetic; it participates in name resolution and code generation,
- each plural name must be globally unique,
- a singular name may have at most one plural companion,
- a plural name cannot also be used as another type's singular name.

Design rule for v1:

- `plural` appears only in type declarations,
- it does not modify field declarations.

### 7.4 Collection Type Semantics

A plural companion name denotes a collection of instances of its associated singular phenotype.

For example, if:

```text
CSVLine plural CSVLines:
    ...
;
```

then:

- `CSVLine` denotes a singular rendered line,
- `CSVLines` denotes a collection whose element type is `CSVLine`.

The language should preserve the distinction between:

- **type plurality**: what a collection of `CSVLine` is called (`CSVLines`), and
- **cardinality**: whether a reference requires one or more values (`+`) or potentially other multiplicities in future versions.

A plural type therefore does **not** by itself imply non-empty cardinality.

### 7.5 Union Types

Union syntax:

```text
{string, int64, MyType}
```

Semantics:

- union members must be distinct after name resolution,
- nested unions are flattened during normalization,
- v1 unions are closed and positional order is preserved only for diagnostics, not semantics,
- plural phenotype names are not valid union members in v1 unless explicitly enabled later.

### 7.6 Cardinality

v1 cardinalities:

- singular required,
- singular optional,
- one-or-more (`+`),
- zero-or-more (`*`).

Cardinality is expressed at the **reference site**.

Examples:

```text
record: required CSVLine
footer: optional CSVLine
lines: required CSVLines+
tags: required Tags*
```

Interpretation:

- `CSVLine` refers to one singular line,
- `CSVLines+` refers to one-or-more collections of lines,
- `Tags*` refers to zero-or-more tags,
- the plural type name communicates collection identity,
- the `+` and `*` operators communicate multiplicity.

`*` pairs naturally with `@ifnotempty` for conditional rendering of potentially empty collections.

This is intentionally orthogonal.

### 7.7 Optionality

Optional fields are fully supported in v1. The type system tracks optionality, and the render model provides declarative directives for handling absent values.

Rules:

- `optional` in the type system denotes a field that may be absent,
- direct `@(field)` interpolation of an optional field is a compile-time error unless it appears inside an `@ifset(field) { ... }` block,
- `@ifset(field) { render_body }` renders its body only when the optional field has a value; inside the block, `@(field)` emits the unwrapped value,
- `@ifnotempty(field) { render_body }` renders its body only when a collection field is non-empty,
- these are declarative guards, not general-purpose control flow.

## 8. Field Declarations

### 8.1 Syntax

```text
field_name: required TypeExpr
field_name: optional TypeExpr
```

### 8.2 Semantics

A field declaration introduces:

- the field name,
- requiredness,
- a normalized type expression,
- source span metadata for diagnostics.

### 8.3 Constraints

v1 constraints are intentionally minimal.

Supported:

- required vs optional,
- scalar vs type reference vs union,
- singular vs collection type reference,
- explicit cardinality at the reference site.

Not supported in v1:

- regex constraints,
- numeric ranges,
- min/max lengths,
- custom predicates.

These belong in a later semantic constraints layer.

## 9. Render Model

### 9.1 Render Body

Each type definition ends with one or more render expressions.

A render body is an ordered sequence of render nodes. Rendering evaluates them left to right and concatenates their emitted text.

### 9.2 Render Expressions

v1 render expressions are:

- string literal text,
- interpolation directive,
- join directive,
- end-of-line directive,
- sequence composition by comma-separated ordering.

### 9.3 Directive Syntax

General directive form:

```text
@name(arg1, arg2, ...)
```

A zero-argument directive still uses parentheses:

```text
@eol()
```

For convenience and readability, a field interpolation shorthand may also be accepted:

```text
@(field_name)
```

This should normalize internally to:

```text
@emit(field_name)
```

The compiler should support the shorthand but represent only canonical directive names in the IR.

## 10. Built-in Directives in v1

### 10.1 `@emit` / `@(field)`

#### Syntax

```text
@(field_name)
@emit(field_name)
```

#### Semantics

- resolves the field name in the current type,
- renders the field value according to its type,
- if the field type is a user-defined singular phenotype type, rendering recursively invokes that type's renderer,
- if the field type is a plural phenotype type, direct `@emit` is invalid in strict v1 and must use `@join`,
- if the field carries one-or-more cardinality, direct `@emit` is invalid and must use `@join`,
- if the field is optional and absent, direct `@emit` is a compile-time error; use `@ifset` instead.

### 10.2 `@join`

#### Syntax

```text
@join(field_name, separator_expr)
```

#### Semantics

- `field_name` must resolve to a joinable field,
- a field is joinable if it references a plural phenotype type or other repeated shape enabled by the compiler,
- `separator_expr` must be either a string literal or a field reference to a singular scalar field,
- each element of the joined field is rendered individually,
- the separator is inserted between consecutive rendered elements,
- no separator appears before the first or after the last element.

Examples:

```text
@join(fields, separator)
@join(lines, "\n")
```

### 10.3 `@eol`

`@eol` is the newline-emitting directive. It is intentionally distinct from `@emit`, which never emits trailing newlines.

#### Syntax

```text
@eol
@eol()
@eol(field_name)
```

#### Semantics

Three forms, all valid:

- `@eol` — bare form, emits `\n`. Most concise.
- `@eol()` — explicit zero-argument form, emits `\n`. Equivalent to `@eol`.
- `@eol(field_name)` — resolves `field_name` to a scalar string, emits that string, then emits a newline.

The field (when provided) must resolve to a singular scalar string in the current type.

The bare form is a parser special case (a directive without parentheses). All three forms normalize to the same `Eol` IR node. As a template language replacement, syntactic economy matters — the less obtrusive the syntax, the better.

### 10.4 String literal render node

A render body may contain string literals directly:

```text
"[", @(value), "]"
```

This is important for punctuation and fixed wrappers.

### 10.5 `@ifset`

#### Syntax

```text
@ifset(field_name) {
    render_expression,
    render_expression
}
```

#### Semantics

- `field_name` must resolve to a field declared `optional`,
- if the field has a value, the enclosed render body is evaluated and its output is emitted,
- if the field is absent (`None`), nothing is emitted,
- within the body, `@(field_name)` emits the unwrapped (non-optional) value,
- the body may contain any valid render expressions including string literals, other directives, and nested `@ifset` or `@ifnotempty` blocks,
- compile-time error if the field is not `optional`.

Example:

```text
CSVFile:
    header: required CSVLine,
    footer: optional CSVLine,
    lines: required CSVLines,
    eol: required string,
    @(header), @eol(eol),
    @join(lines, eol),
    @ifset(footer) { @eol(eol), @(footer) }
;
```

### 10.6 `@ifnotempty`

#### Syntax

```text
@ifnotempty(field_name) {
    render_expression,
    render_expression
}
```

#### Semantics

- `field_name` must resolve to a field referencing a plural type or carrying `+` cardinality,
- if the collection is non-empty, the enclosed render body is evaluated and its output is emitted,
- if the collection is empty (possible for optional collections or future `*` cardinality), nothing is emitted,
- within the body, `@join(field_name, separator)` and other collection directives behave normally,
- compile-time error if the field does not reference a collection type.

## 11. Name Resolution

### 11.1 Type names

Type references are resolved against the set of type definitions in the compilation unit.

v1 rule:

- all referenced singular and plural names must be defined exactly once in the same file.

### 11.2 Singular/plural resolution

The compiler should maintain a symbol table that tracks:

- singular phenotype names,
- plural companion names,
- the mapping from plural name to singular element type.

Example resolution table:

```text
CSVLine   -> singular type definition
CSVLines  -> plural companion of CSVLine
```

### 11.3 Field names

Field references inside directives resolve only within the current type.

Nested field-path expressions like `header.title` are out of scope for v1.

### 11.4 Duplicate names

Compiler errors:

- duplicate singular type name,
- duplicate plural type name,
- singular name colliding with an existing plural name,
- plural name colliding with an existing singular name,
- duplicate field name within a type,
- unknown field reference,
- unknown type reference.

## 12. Formal Grammar Sketch

This is a near-EBNF sketch intended to guide Rustemo grammar work. It is not yet a complete parser grammar, but it defines the intended structure.

```ebnf
file                = { type_definition } ;

type_definition     = type_header ":" type_body ";" ;
type_header         = identifier [ "plural" identifier ] ;

type_body           = body_item { "," body_item } ;

body_item           = field_declaration | render_expression ;

field_declaration   = identifier ":" requiredness type_expr ;
requiredness        = "required" | "optional" ;

type_expr           = cardinalized_type | union_type | named_type ;
cardinalized_type   = named_type ( "+" | "*" ) ;
union_type          = "{" type_expr_member { "," type_expr_member } "}" ;
type_expr_member    = named_type ;
named_type          = identifier ;

render_expression   = string_literal | block_directive | directive | bare_directive ;

block_directive      = "@" identifier "(" identifier ")" "{" { render_expression } "}" ;
directive           = "@" identifier "(" [ argument_list ] ")"
                    | "@(" identifier ")" ;
bare_directive      = "@" "eol" ;   /* special case: @eol without parentheses */
argument_list       = expression_argument { "," expression_argument } ;
expression_argument = identifier | string_literal ;

identifier          = /* lexer-defined */ ;
string_literal      = /* lexer-defined */ ;
```

Implementation note: primitive types may be tokenized as identifiers and later recognized during semantic analysis, which simplifies the grammar.

## 13. Abstract Syntax Tree

The parser should produce a syntax tree with source spans preserved. A practical AST model:

```text
FileAst
  - definitions: Vec<TypeDefAst>

TypeDefAst
  - singular_name: Ident
  - plural_name: Option<Ident>
  - items: Vec<TypeItemAst>
  - span: Span

TypeItemAst
  - Field(FieldDeclAst)
  - Render(RenderExprAst)

FieldDeclAst
  - requiredness: RequirednessAst
  - name: Ident
  - ty: TypeExprAst
  - span: Span

TypeExprAst
  - Named(Ident)
  - Union(Vec<TypeExprAst>)
  - Cardinalized(Box<TypeExprAst>, CardinalityAst)

CardinalityAst
  - OneOrMore
  - ZeroOrMore

RenderExprAst
  - StringLiteral(String)
  - Directive(DirectiveAst)

DirectiveAst
  - name: Ident
  - args: Vec<ExprArgAst>
  - shorthand: bool
  - span: Span

ExprArgAst
  - Ident(Ident)
  - StringLiteral(String)
```

The AST should remain close to source structure. Semantic normalization belongs in the IR stage.

## 14. Intermediate Representation

The IR should remove syntax sugar and resolve names as much as possible.

Recommended IR:

```text
PhenotypeModule
  - types: Vec<PhenotypeType>

PhenotypeType
  - id: TypeId
  - singular_name: String
  - plural_name: Option<String>
  - fields: Vec<FieldDef>
  - render: Vec<RenderNode>

FieldDef
  - id: FieldId
  - name: String
  - requiredness: Requiredness
  - cardinality: Cardinality
  - ty: ValueType

Requiredness
  - Required
  - Optional

Cardinality
  - One
  - OneOrMore
  - ZeroOrMore

ValueType
  - Primitive(PrimitiveType)
  - UserSingular(TypeId)
  - UserPlural { collection_of: TypeId }
  - Union(Vec<ValueType>)

PrimitiveType
  - String
  - Int64
  - Real64
  - Bool
  - Date
  - Time
  - DateTime

RenderNode
  - Text(String)
  - Emit(FieldId)
  - Join { field: FieldId, separator: SeparatorExpr }
  - Eol { field: Option<FieldId> }
  - IfSet { field: FieldId, body: Vec<RenderNode> }
  - IfNotEmpty { field: FieldId, body: Vec<RenderNode> }

SeparatorExpr
  - Literal(String)
  - Field(FieldId)
```

`Eol` is a first-class render node:
- `Eol { field: None }` corresponds to `@eol()` — emits `\n`.
- `Eol { field: Some(id) }` corresponds to `@eol(field)` — emits the field value then `\n`.

## 15. Normalization Rules

The AST-to-IR pass should apply the following normalizations.

### 15.1 Directive canonicalization

- `@(field)` becomes `@emit(field)`.
- `@eol()` becomes `Eol { field: None }`.
- `@eol(field)` becomes `Eol { field: Some(field_id) }`.
- `@eol` is **not** lowered to `@emit`; it is a distinct render node.

### 15.2 Union flattening

Nested unions are flattened.

### 15.3 Primitive recognition

Identifiers matching primitive type names are lowered to `PrimitiveType` rather than `UserSingular(TypeId)`.

### 15.4 Singular/plural normalization

- a type definition with `Singular plural Plural` stores both names on the same `PhenotypeType`,
- the compiler records a mapping from `Plural` to the `Singular` type id,
- a field typed as `PluralName` lowers to `ValueType::UserPlural { collection_of: SingularTypeId }`.

### 15.5 Cardinality normalization

Cardinality operators belong to the reference site, not the type definition.

For example:

- `CSVLine` lowers to `UserSingular(CSVLineId)` with `Cardinality::One`,
- `CSVLines+` lowers to `UserPlural { collection_of: CSVLineId }` with `Cardinality::OneOrMore`,
- `Tags*` lowers to `UserPlural { collection_of: TagId }` with `Cardinality::ZeroOrMore`.

### 15.6 Field/type resolution

Unknown names are rejected before code generation.

## 16. Validation Phases

The compiler should validate incrementally.

### 16.1 Parse validation

Handled by grammar:

- malformed tokens,
- unbalanced punctuation,
- structurally invalid constructs.

### 16.2 Symbol validation

- duplicate singular type definitions,
- duplicate plural type definitions,
- singular/plural name collisions,
- duplicate field definitions,
- unknown type references,
- unknown field references.

### 16.3 Type validation

- plural companion names must refer only to the type they are declared on,
- plural type references must resolve to declared plural companions,
- union members valid,
- illegal direct emit of plural fields,
- separator field must be singular scalar string,
- direct emit of optional fields outside `@ifset` rejected,
- `@ifset` on non-optional field rejected,
- `@ifnotempty` on non-collection field rejected.

### 16.4 Render validation

- each directive must have the correct arity,
- each directive argument kind must be valid,
- all field references must be resolvable,
- render body must be non-empty.

### 16.5 Generation validation

- all types must be generatable to Rust,
- unions must map to an enum shape,
- recursive types should be detected and either rejected or handled explicitly.

### 16.6 Recursion policy

v1 recommended rule:

- reject cyclic type graphs unless and until a deliberate ownership/rendering model is defined.

This keeps the first compiler and generated code substantially simpler.

## 17. Diagnostics

Compiler diagnostics should be a first-class output.

Each error should include:

- severity,
- concise summary,
- source span,
- expanded explanation,
- when possible, a fix suggestion.

Examples:

- `Unknown field 'records' in @join(records, separator)`
- `Field 'lines' references plural type 'CSVLines' and cannot be rendered with @emit; use @join(lines, ... )`
- `Plural name 'CSVLines' already declared by phenotype 'CSVLine'`
- `Unknown plural type 'CSVLines'`
- `Optional field 'footer' cannot be directly emitted in v1`
- `Duplicate type definition 'CSVLine'`

## 18. Compiler Pipeline

Recommended compiler phases:

1. lexing,
2. parsing into AST,
3. AST validation for local structural sanity,
4. symbol collection,
5. name resolution,
6. AST-to-IR normalization,
7. semantic validation over IR,
8. Rust code generation,
9. optional documentation or debug IR emission.

Recommended CLI subcommands in the long run:

```bash
phenotyper check example.pht
phenotyper build example.pht --out src/generated
phenotyper dump-ast example.pht
phenotyper dump-ir example.pht
```

## 19. Rust Code Generation

### 19.1 Output shape

For each phenotype type, code generation should produce:

- a Rust data type representing the singular instance,
- when a plural companion exists, a dedicated Rust wrapper type representing the collection instance,
- a builder type,
- validation methods if needed,
- a render method or trait implementation.

### 19.2 Generated data types

Illustrative shape:

```rust
pub struct CsvLine {
    pub fields: CsvFieldValues,
    pub separator: String,
}

pub struct CsvLines {
    items: Vec<CsvLine>,
}

impl CsvLines {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<CsvLine>) -> Self {
        Self { items }
    }

    pub fn into_vec(self) -> Vec<CsvLine> {
        self.items
    }

    pub fn as_slice(&self) -> &[CsvLine] {
        &self.items
    }

    pub fn iter(&self) -> std::slice::Iter<'_, CsvLine> {
        self.items.iter()
    }
}

impl From<Vec<CsvLine>> for CsvLines {
    fn from(items: Vec<CsvLine>) -> Self {
        Self::from_vec(items)
    }
}

impl From<CsvLines> for Vec<CsvLine> {
    fn from(value: CsvLines) -> Self {
        value.into_vec()
    }
}
```

Plural companions always generate dedicated Rust wrapper types. Their canonical internal storage is `Vec<SingularType>`, but the wrapper must expose ergonomic conversions and collection-like APIs so the vector representation remains easy to access without erasing the domain-level type. At minimum, generated plural wrappers should support `from_vec`, `into_vec`, slice or iterator access, and `From<Vec<T>>` / `Into<Vec<T>>` conversions. The wrapper's storage should remain private in v1 so that future invariants and collection-level behavior can be added safely.

For required singular fields, plain Rust fields are acceptable.

For optional fields:

```rust
pub footer: Option<CsvLine>
```

For unions, Rust enums should be generated.

### 19.3 Builder API conventions

Recommended builder pattern:

- `TypeNameBuilder::new(required_args...)` if all required singular scalar args are convenient,
- or a uniform empty builder with mandatory setters enforced at `build()` time,
- plural fields support `push_*` and `extend_*`,
- builder methods consume `self` or use `&mut self`; choose one style consistently.

Recommended v1 choice:

- `&mut self` setters for ergonomics,
- `build() -> Result<TypeName, BuildError>`.

Illustrative API:

```rust
let mut builder = CsvFileBuilder::new();
builder.header(header_line);
builder.lines(csv_lines);
builder.eol("\n".to_string());
let file = builder.build()?;
let rendered = file.render();
```

### 19.4 Rendering trait

The `Render` trait uses `render_into` as the core method for zero-allocation recursive rendering, with a convenience `render()` wrapper:

```rust
pub trait Render {
    fn render_into(&self, out: &mut String);

    fn render(&self) -> String {
        let mut buf = String::new();
        self.render_into(&mut buf);
        buf
    }
}
```

All generated phenotype structs implement `Render` by implementing `render_into`.

Plural companion structs also implement `Render` by joining their singular elements according to the phenotype definition that references them.

### 19.5 Union mapping

A field with type `{string, int64}` should become a generated enum such as:

```rust
pub enum CsvFieldValue {
    String(String),
    Int64(i64),
}
```

Field names and generated enum names should follow deterministic naming rules.

### 19.6 Validation placement

Two layers are recommended:

- compile-time validation for language definition errors,
- runtime builder validation for missing required values or invalid multiplicity.

Example runtime checks:

- required field not set,
- `OneOrMore` field empty.

### 19.7 Naming strategy

When a phenotype declares:

```text
CSVLine plural CSVLines:
```

then Rust generation should preserve both names as closely as idiomatic Rust allows:

- `CsvLine`
- `CsvLines`
- `CsvLineBuilder`
- `CsvLinesBuilder` for the dedicated collection wrapper.

This is one of the direct benefits of having `plural` in the source language.

## 20. Runtime Semantics

### 20.1 Rendering scalars

Suggested default scalar rendering:

- `string`: emitted as-is,
- `int64`: decimal representation,
- `real64`: Rust default or chosen normalized format,
- `bool`: `true` / `false`,
- `date`, `time`, `datetime`: emitted according to the Rust backing type's canonical string form.

Precise formatting policy for temporal types should be fixed explicitly, ideally ISO-oriented.

### 20.2 Rendering user-defined singular types

Rendered recursively via generated `Render` implementation.

### 20.3 Rendering plural types

Plural types are joinable collection values.

In strict v1:

- they are rendered through `@join`,
- they are not directly emitted with `@emit`,
- the renderer iterates over the singular element type associated with the plural name.

### 20.4 Escaping

Phenotyper v1 does not impose automatic escaping rules. Values are rendered literally. Escaping-sensitive formats are the responsibility of the phenotype design or later directive extensions.

This is intentionally simple but should be clearly documented.

## 21. Minimum Viable Standard Library

Phenotyper v1 should resist the urge for a large directive library.

Recommended built-ins only:

- `@emit`
- `@join`
- `@eol` if kept distinct
- `@ifset`
- `@ifnotempty`

Potential future directives, but not v1:

- `@indent`
- `@surround`
- `@trim`
- `@escape_csv`
- `@escape_html`

## 22. Example: Normalized CSV Model

Source:

```text
CSVFieldValue plural CSVFieldValues:
    value: required {int64, real64, string, date, time, datetime},
    @(value)
;

CSVLine plural CSVLines:
    fields: required CSVFieldValues,
    separator: required string,
    @join(fields, separator)
;

CSVFile:
    header: required CSVLine,
    lines: required CSVLines,
    eol: required string,
    @(header), @eol(eol), @join(lines, eol)
;
```

Observations:

- `CSVFieldValues` is the named plural companion of `CSVFieldValue`.
- `CSVLine.fields` references the plural collection type rather than encoding plurality only with punctuation.
- `CSVLines` is the named plural companion of `CSVLine`.
- `CSVFile.lines` references `CSVLines`, which can then be joined with `@join(lines, eol)`.
- If non-empty cardinality must be enforced explicitly, the reference may be written as `CSVLines+`.

This is a strong v1 test case because it exercises:

- singular/plural type naming,
- unions,
- collection rendering through `@join`,
- recursive rendering through composed types,
- explicit separator management.

## 23. CLI Contract for v1 Compiler

Minimum v1 compiler CLI:

```bash
phenotyper check path/to/file.pht
phenotyper build path/to/file.pht --out path/to/generated
```

Recommended optional flags:

```bash
phenotyper check file.pht --json
phenotyper dump-ast file.pht
phenotyper dump-ir file.pht
```

Exit code policy:

- `0` on success,
- non-zero on validation or generation failure.

## 24. Testing Strategy

### 24.1 Parser tests

- golden tests for valid syntax,
- negative tests for malformed syntax,
- comment and whitespace cases,
- singular/plural header parsing cases.

### 24.2 Semantic tests

- duplicate names,
- singular/plural collisions,
- unknown references,
- invalid directive arity,
- illegal direct emit of plural fields,
- invalid separator types,
- optional emit rejection.

### 24.3 Codegen tests

- snapshot tests of generated Rust,
- compile-tests for generated code,
- runtime rendering tests,
- naming tests for singular/plural Rust output.

### 24.4 End-to-end tests

- CSV example,
- one structured prompt example,
- at least one union-heavy example.

## 25. Extension Points Beyond v1

The v1 design should intentionally preserve room for later additions.

### 25.1 Parser generation

The IR should be rich enough that future work can derive parsers from at least a constrained subset of phenotype definitions.

### 25.2 Imports and modules

Multi-file organization is deferred but should not be blocked by v1 internals.

### 25.3 Semantic constraints

Future field-level constraints may include:

- regex patterns,
- value ranges,
- length constraints,
- domain-specific validation directives.

### 25.4 Optional-aware rendering

v1 includes `@ifset` and `@ifnotempty` for declarative conditional rendering. Later versions may extend this with richer constructs such as `@ifelse` or pattern-matching blocks, if warranted by real-world use cases.

### 25.5 Template conversion CLI

The future template conversion feature should target the same IR where practical.

A plausible strategy:

- foreign template parser -> template IR -> Phenotyper IR subset -> `.pht` emission + migration report.

Design implication:

The v1 IR should remain clean and structural so that imported templates can map into it where possible.

## 26. Open Design Decisions Still Worth Resolving

The following should be decided early in implementation:

1. whether `optional` is fully enabled in v1 or merely parsed and partially validated,
2. whether `@eol` survives as a distinct directive or lowers to `@emit`,
3. whether `*` repetition is accepted in v1,
4. whether builder setters consume `self` or use `&mut self`,
5. which Rust types back `date`, `time`, and `datetime`,
6. whether recursive type graphs are rejected outright in the first release,
7. whether primitive type names are fully reserved lexically or resolved semantically,
8. whether plural wrappers implement `Deref<[T]>` in v1 or only explicit collection methods and conversions.

## 27. Recommended v1 Decisions

To keep implementation disciplined, the recommended choices are:

- support `required` and `optional` in syntax, with `@ifset` and `@ifnotempty` for conditional rendering,
- reserve `plural` for phenotype declarations only,
- always generate dedicated Rust wrapper types for plural companions, while exposing ergonomic `Vec<T>` conversions and collection-style APIs,
- support `+` only for explicit cardinality in v1,
- canonicalize `@(field)` to `@emit(field)`,
- treat `@join` as the repeated-render mechanism,
- keep `@eol(field)` explicit,
- reject cyclic type graphs,
- generate Rust builders using `&mut self` setters and `build()` validation,
- keep escaping out of the core semantics.

## 28. Implementation Checklist

A practical implementation sequence:

1. define lexer tokens,
2. implement Rustemo grammar including singular/plural type headers,
3. produce AST with spans,
4. implement symbol table and singular/plural name resolution,
5. implement IR lowering and normalization,
6. implement semantic validator,
7. implement Rust type mapping,
8. implement Rust renderer generation,
9. implement builder generation,
10. add `check` and `build` CLI commands,
11. add golden tests and end-to-end examples.

## 29. Summary

Phenotyper v1 should be intentionally small, structural, and compiler-friendly. Its job is to define renderable artifact families with enough precision to generate reliable Rust construction and rendering code.

A central v1 choice is that `plural` belongs in phenotype declarations, where it names the canonical collection companion of a singular phenotype. Cardinality operators such as `+` then remain available at reference sites to express multiplicity constraints.

The right v1 is not the most expressive one. It is the smallest one that cleanly proves the central claim:

> a human-readable artifact definition can serve as the single source of truth for generated builders, deterministic renderers, and structural validation.

That foundation will make later work on parser generation, richer directives, imports, and template conversion substantially more credible.


## 20. Namespace-level Enum Types

Phenotyper v1 supports enums as explicit namespace-level reusable `type` declarations. The normative enum form is:

```phenotyper
type Visibility: [public, protected, private];
```

Enum semantics in v1:

- an enum is a closed set of symbolic values,
- enum members are identifiers, not string literals,
- enum members must be unique within the declaration,
- enums are declared only at namespace scope in v1,
- nested `type` or enum declarations inside phenotype bodies are not part of v1 and must be rejected,
- enum values render by their declared source spelling unless later metadata changes that behavior.

Enums participate in name resolution exactly like other namespace-level reusable types and may be referenced from field declarations:

```phenotyper
namespace aivolution/lang/model;

type Visibility: [public, protected, private];

Method plural Methods:
    visibility: required Visibility,
    @(visibility),
    @" ",
    @(name)
;
```

Rust generation guidance:

```rust
pub enum Visibility {
    Public,
    Protected,
    Private,
}
```

The Rust enum variant names are derived deterministically from the declared enum members, while default rendering still emits the original declared spellings `public`, `protected`, and `private`.

This section is normative and supersedes any earlier ambiguity in this draft about whether enums may be declared inside phenotype definitions. In v1 they are namespace-level only.
