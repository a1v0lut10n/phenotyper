# REQ-COMP — Compiler Pipeline Requirements

## Status
v1 — Derived from `phenotyper_v1_language_and_compiler_spec.md` sections 12–18.

---

## REQ-COMP-001 — Compiler pipeline phases

The v1 compiler shall implement the following phases in order:

| Phase | Name | Description |
|-------|------|-------------|
| 1 | Lexing | Tokenize source into lexical tokens |
| 2 | Parsing | Produce an AST from tokens using Rustemo |
| 3 | AST validation | Local structural sanity checks |
| 4 | Symbol collection | Build symbol table of types and fields |
| 5 | Name resolution | Resolve all type and field references |
| 6 | AST-to-IR normalization | Lower surface syntax to clean IR |
| 7 | Semantic validation | Validate semantic rules over the IR |
| 8 | Rust code generation | Emit generated Rust source |
| 9 | Debug emission *(optional)* | Dump AST or IR for diagnostics |

---

## REQ-COMP-002 — Source extraction (Markdown container)

When the source container is `.md`:
- Extract all `pht` fenced code blocks in document order.
- Build a **source map** that tracks the offset of each extracted block relative to the original markdown file.
- All subsequent phases operate on the concatenated extracted content.
- Diagnostics reference the original `.md` line and column via the source map.

---

## REQ-COMP-003 — Lexer tokens

The lexer shall produce tokens for at minimum:

| Token | Pattern |
|-------|---------|
| Identifier | `[A-Za-z_][A-Za-z0-9_]*` |
| StringLiteral | `"..."` with escape sequences |
| Colon | `:` |
| Comma | `,` |
| Semicolon | `;` |
| LeftBrace | `{` |
| RightBrace | `}` |
| LeftBracket | `[` |
| RightBracket | `]` |
| LeftParen | `(` |
| RightParen | `)` |
| At | `@` |
| Plus | `+` |
| Star | `*` *(reserved)* |
| Slash | `/` (for namespace paths) |
| LineComment | `// ...` (discarded) |
| BlockComment | `/* ... */` (discarded) |
| Whitespace | (discarded) |

---

## REQ-COMP-004 — Parser and grammar

- The parser shall be implemented using **Rustemo**.
- The grammar shall conform to the EBNF sketch in spec section 12.
- The parser shall produce a syntax tree with **source spans preserved** on all nodes.

### Grammar summary (near-EBNF):

```ebnf
file                = { namespace_decl } { uses_decl } { top_level_decl } ;
namespace_decl      = "namespace" namespace_path ";" ;
uses_decl           = "uses" namespace_path ";" ;
namespace_path      = identifier { "/" identifier } ;
top_level_decl      = type_alias | enum_decl | type_definition ;
type_alias          = "type" identifier ":" type_expr ";" ;
enum_decl           = "type" identifier ":" "[" identifier { "," identifier } "]" ";" ;
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
```

---

## REQ-COMP-005 — Abstract Syntax Tree (AST)

The AST shall model the following node types:

```
FileAst
  - namespace: NamespaceAst
  - uses: Vec<UsesAst>
  - type_aliases: Vec<TypeAliasAst>
  - enum_decls: Vec<EnumDeclAst>
  - definitions: Vec<TypeDefAst>

NamespaceAst
  - path: Vec<Ident>
  - span: Span

UsesAst
  - path: Vec<Ident>
  - span: Span

TypeAliasAst
  - name: Ident
  - target: TypeExprAst
  - span: Span

EnumDeclAst
  - name: Ident
  - members: Vec<Ident>
  - span: Span

TypeDefAst
  - singular_name: Ident
  - plural_name: Option<Ident>
  - items: Vec<TypeItemAst>
  - span: Span

TypeItemAst
  - Field(FieldDeclAst)
  - Render(RenderExprAst)

FieldDeclAst
  - name: Ident
  - requiredness: RequirednessAst
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
  - StringLiteral(String, Span)
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

---

## REQ-COMP-006 — Intermediate Representation (IR)

The IR shall desugare the AST, resolve names, and normalize constructs:

```
PhenotypeModule
  - namespace: String
  - types: Vec<PhenotypeType>
  - enums: Vec<EnumType>
  - type_aliases: Vec<TypeAlias>

PhenotypeType
  - id: TypeId
  - singular_name: String
  - plural_name: Option<String>
  - fields: Vec<FieldDef>
  - render: Vec<RenderNode>

FieldDef
  - id: FieldId
  - name: String
  - requiredness: Requiredness { Required, Optional }
  - cardinality: Cardinality { One, OneOrMore, ZeroOrMore }
  - ty: ValueType

ValueType
  - Primitive(PrimitiveType)
  - UserSingular(TypeId)
  - UserPlural { collection_of: TypeId }
  - Enum(EnumId)
  - TypeAlias(AliasId)
  - Union(Vec<ValueType>)

PrimitiveType
  - String, Int64, Real64, Bool, Date, Time, DateTime

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

EnumType
  - id: EnumId
  - name: String
  - members: Vec<String>
```

---

## REQ-COMP-007 — Normalization rules (AST → IR)

| Rule | Description |
|------|-------------|
| Directive canonicalization | `@(field)` becomes `Emit(field_id)` |
| Eol lowering | `@eol()` becomes `Eol { field: None }`, `@eol(field)` becomes `Eol { field: Some(field_id) }` — **not** lowered to `Emit` |
| Union flattening | Nested unions are recursively flattened |
| Primitive recognition | Primitive type names are lexically reserved tokens; the parser produces `PrimitiveType` nodes directly |
| Singular/plural normalization | Plural companions stored on the same `PhenotypeType`; field typed as `PluralName` lowers to `UserPlural { collection_of: SingularTypeId }` |
| Cardinality normalization | `+` operator lowered to `Cardinality::OneOrMore`, `*` lowered to `Cardinality::ZeroOrMore` |
| Field/type resolution | All unknown names rejected before code generation |

---

## REQ-COMP-008 — Symbol table

The compiler shall maintain a symbol table tracking:

| Entry | Information |
|-------|-------------|
| Singular phenotype names | `TypeId`, field list |
| Plural companion names | Mapped to singular `TypeId` |
| Enum names | `EnumId`, member list |
| Type alias names | `AliasId`, target type |
| Field names per type | `FieldId`, type, requiredness |

---

## REQ-COMP-009 — Validation phases

### Parse validation (grammar-level)
- Malformed tokens.
- Unbalanced punctuation.
- Structurally invalid constructs.

### Symbol validation
- Duplicate singular type definitions → **error**.
- Duplicate plural type definitions → **error**.
- Singular/plural name collisions → **error**.
- Duplicate field definitions within a type → **error**.
- Unknown type references → **error**.
- Unknown field references → **error**.
- Duplicate enum member names → **error**.

### Type validation
- Plural companion names must refer only to the type they are declared on.
- Plural type references must resolve to declared plural companions.
- Union members must be valid.
- Direct `@emit` of plural fields → **error** (must use `@join`).
- Separator field must be a singular scalar string.
- Direct `@emit` of optional fields outside `@ifset` → **error**.
- `@ifset` on a non-optional field → **error**.
- `@ifnotempty` on a non-collection field → **error**.

### Render validation
- Each directive must have the correct arity.
- Each directive argument kind must be valid.
- All field references must be resolvable.
- Render body must be non-empty.

### Generation validation
- All types must map to valid Rust types.
- Unions must map to an enum shape.
- Recursive/cyclic type graphs → **error** in v1.

---

## REQ-COMP-010 — Diagnostics

Each diagnostic shall include:

| Field | Description |
|-------|-------------|
| Severity | error / warning / info |
| Summary | Concise human-readable message |
| Source span | File, line, column (original source coordinates) |
| Explanation | Expanded description of the issue |
| Fix suggestion | When feasible |

Example diagnostics:
- `Unknown field 'records' in @join(records, separator)`
- `Field 'lines' references plural type 'CSVLines' and cannot be rendered with @emit; use @join(lines, ...)`
- `Plural name 'CSVLines' already declared by phenotype 'CSVLine'`
- `Optional field 'footer' cannot be directly emitted in v1`
- `Duplicate type definition 'CSVLine'`

---

## REQ-COMP-011 — Recursion policy

v1 shall **reject** cyclic type graphs.

This keeps the compiler and generated code substantially simpler for the first release.

---

## REQ-COMP-012 — Name resolution rules

- Every file belongs to exactly one namespace.
- Local names must be unique within that namespace.
- `uses` brings all declarations from a namespace into scope.
- Local declarations shadow imported ones → **warning**.
- Ambiguous imported names → **error** unless fully qualified.
- Duplicate names within the same namespace → **error**.
- Field references in directives resolve only within the current type.
- Nested field-path expressions (e.g., `header.title`) are out of scope for v1.
