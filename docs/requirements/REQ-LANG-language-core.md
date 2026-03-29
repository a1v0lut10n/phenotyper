# REQ-LANG — Core Language Requirements

## Status
v1 — Derived from `phenotyper_v1_language_and_compiler_spec.md` and `phenotyper_design_memo.md`.

---

## REQ-LANG-001 — Source containers

Phenotyper v1 shall support two normative source containers:

| Container | Extension | Behavior |
|-----------|-----------|----------|
| Markdown  | `.md`     | `pht` fenced code blocks are extracted in document order; surrounding markdown is documentation only. |
| Pure      | `.pht`    | Parsed directly as the core language. |

Both containers shall compile to **the same core language model**.

All diagnostics shall reference the **exact original line and column** in the author's source file, regardless of container type.

---

## REQ-LANG-002 — Namespaces

Each source file shall declare exactly one namespace:

```pht
namespace aivolution/format/csv;
```

- The `/` character is the namespace separator.
- Namespace paths do **not** use a leading `/`.
- Namespace maps to Rust module path (e.g., `aivolution/format/csv` → `aivolution::format::csv`).
- Namespace declaration is mandatory and must appear before all other declarations.

---

## REQ-LANG-003 — Imports (`uses`)

```pht
uses aivolution/core/time;
```

- `uses` brings all declarations from the referenced namespace into unqualified scope.
- Local declarations shadow imported ones — the compiler shall emit a **warning**.
- Ambiguous imported names are **hard errors** unless fully qualified.

---

## REQ-LANG-004 — Reusable named types (`type`)

```pht
type ScalarValue: {int64, real64, string, Date, Time, Datetime};
type CsvText: string;
```

- `type Name: TypeExpression;` declares a namespace-level reusable type.
- Type aliases make a name an alias for an existing type or union.

---

## REQ-LANG-005 — Enum types

```pht
type Visibility: [public, protected, private];
```

- Enums use square brackets `[ ]` to distinguish from union types `{ }`.
- Enums are a closed set of symbolic values; members are identifiers, not string literals.
- Enum members must be unique within the declaration.
- Enums are declared at namespace scope only in v1; nested enums inside phenotype bodies shall be rejected.
- Enum values render by their declared source spelling (default).
- Enums participate in name resolution like other namespace-level types.

---

## REQ-LANG-006 — Primitive scalar types

Phenotyper v1 shall support the following primitive scalar types:

| Type       | Semantic category            |
|------------|------------------------------|
| `string`   | Textual data                 |
| `int64`    | 64-bit integer               |
| `real64`   | 64-bit floating point        |
| `bool`     | Boolean                      |
| `date`     | Calendar date                |
| `time`     | Time of day                  |
| `datetime` | Combined date and time       |

These are semantic categories. Concrete Rust backing types are determined during code generation.

---

## REQ-LANG-007 — Phenotype declarations (singular and plural)

A phenotype type declaration has this canonical form:

```pht
SingularName plural PluralName:
    field_declarations,
    render_expressions
;
```

- `:` introduces the type body.
- `,` separates declarations and render expressions within the body.
- `;` terminates the type definition.
- The `plural` keyword and plural companion name are optional.

Plural semantics:
- The singular name denotes one instance.
- The plural name denotes the collection phenotype.
- Each plural name must be globally unique.
- A singular name may have at most one plural companion.
- A plural name cannot also be used as another type's singular name.
- `plural` appears only in type declarations, not in field declarations.

---

## REQ-LANG-008 — Field declarations

```pht
field_name: required TypeExpr
field_name: optional TypeExpr
```

- The field name comes first, followed by `:`, then the requiredness keyword (`required` or `optional`), then the type expression.
- This makes the field name the first thing a reader sees, improving scanability.
- A field declaration introduces the field name, requiredness, a normalized type expression, and source-span metadata.

---

## REQ-LANG-009 — Type expressions and references

A field may reference:
- a primitive type,
- a user-defined singular phenotype type,
- a user-defined plural phenotype type,
- a union of types (using `{ }` syntax),
- a named enum type (using `[ ]` syntax at declaration, referenced by name),
- a reusable type alias,
- a cardinality-annotated form where permitted.

---

## REQ-LANG-010 — Union types

```pht
{string, int64, MyType}
```

- Union members must be distinct after name resolution.
- Nested unions are flattened during normalization.
- Unions are closed; positional order is preserved only for diagnostics.
- Plural phenotype names are not valid union members in v1.

---

## REQ-LANG-011 — Cardinality

v1 cardinalities at the **reference site**:

| Form          | Meaning          |
|---------------|------------------|
| (default)     | Singular         |
| `+`           | One-or-more      |
| `*`           | Zero-or-more     |

Combined with `required`/`optional`:
- `field: required CSVLine` → exactly one.
- `field: optional CSVLine` → zero or one.
- `field: required CSVLines+` → one-or-more collection.
- `field: required CSVLines*` → zero-or-more collection.

`*` pairs naturally with `@ifnotempty` for conditional rendering of potentially empty collections.

Cardinality is orthogonal to plural type naming.

---

## REQ-LANG-012 — Optionality and conditional rendering in v1

- `optional` is fully supported in the v1 type system and render model.
- Direct `@(field)` interpolation of an optional field **outside** an `@ifset` block is a **compile-time error**.
- The `@ifset(field) { ... }` directive renders its body only when the optional field has a value. Inside the block, `@(field)` emits the unwrapped value.
- The `@ifnotempty(field) { ... }` directive renders its body only when a collection field (plural type or `+` cardinality) is non-empty.
- These directives are declarative guards — they do not introduce general-purpose `if/else` or boolean expressions.

---

## REQ-LANG-013 — Render body

Each type definition ends with one or more render expressions. The render body is an ordered sequence of render nodes evaluated left-to-right, concatenating emitted text.

v1 render expressions:
- String literal text (`"text"`)
- Interpolation directive (`@(field)` / `@emit(field)`)
- Join directive (`@join(field, separator)`)
- End-of-line directive (`@eol(field)`)
- Conditional optional directive (`@ifset(field) { ... }`)
- Conditional non-empty directive (`@ifnotempty(field) { ... }`)
- Field-path emission (`@(field/subfield)`) *(per README; nested paths deferred per spec)*
- Sequence composition by comma-separated ordering

---

## REQ-LANG-014 — Directive syntax

General form:

```pht
@name(arg1, arg2, ...)
@name()            // zero-argument
@(field_name)      // shorthand for @emit(field_name)
```

The shorthand `@(field)` shall normalize to `@emit(field)` in the IR.

---

## REQ-LANG-015 — Built-in directives

### `@emit` / `@(field)`
- Resolves field name in the current type and renders its value.
- Recursive rendering for user-defined singular types.
- **Invalid** for plural types or cardinality `+` fields — must use `@join`.
- **Invalid** for absent optional values in v1.

### `@join(field, separator)`
- Field must be joinable (plural type or repeated shape).
- Separator must be a string literal or field reference to a singular scalar string.
- Elements rendered individually; separator inserted between consecutive elements only.

### `@eol`, `@eol()`, and `@eol(field)`
- `@eol` is the newline-emitting directive, distinct from `@emit`.
- Three forms, all valid:
  - `@eol` — bare form, emits `\n`. Most concise.
  - `@eol()` — explicit zero-argument form, emits `\n`. Equivalent to `@eol`.
  - `@eol(field)` — emits the field value as a scalar string, followed by `\n`.
- The field (when provided) must resolve to a singular scalar string.
- `@emit` and `@(field)` never emit trailing newlines; `@eol` provides explicit newline control.
- The bare form is a parser special case (a directive without parentheses). All three normalize to the same `Eol` IR node.

### `@ifset(field) { render_body }`
- Field must be declared `optional`.
- If the field has a value, renders the enclosed render body.
- If the field is absent (`None`), emits nothing.
- Within the body, `@(field)` emits the unwrapped (non-optional) value.
- The body may contain any valid render expressions: string literals, `@(field)`, `@join`, `@eol`, nested `@ifset`, etc.
- Compile-time error if the field is not `optional`.

### `@ifnotempty(field) { render_body }`
- Field must reference a plural type or have `+` cardinality.
- If the collection is non-empty, renders the enclosed render body.
- If the collection is empty (only possible for optional collections or future `*` cardinality), emits nothing.
- Within the body, `@join(field, separator)` and other collection directives behave normally.
- Compile-time error if the field does not reference a collection type.

### String literal render node
- Render body may contain string literals directly: `"[", @(value), "]"`.

---

## REQ-LANG-016 — Identifiers

- Must begin with `[A-Za-z_]`.
- May continue with `[A-Za-z0-9_]`.
- Case-sensitive.
- Convention: `PascalCase` for type names, `snake_case` or `lowerCamelCase` for field names (v1 compiler accepts either).

---

## REQ-LANG-017 — Keywords

Reserved words in v1: `required`, `optional`, `plural`, `namespace`, `uses`, `type`, `string`, `int64`, `real64`, `bool`, `date`, `time`, `datetime`.

Primitive type names are lexically reserved and cannot be used as user-defined type or field names.

---

## REQ-LANG-018 — Literals

v1 supports string literals with double quotes: `"text"`.

Numeric literals are not needed in the language itself (numbers are runtime field values).

---

## REQ-LANG-019 — Comments

- Line comment: `// ...`
- Block comment: `/* ... */`
- Comments are ignored by the parser.
- Comments are available only in `.pht` files; in `.md` files, markdown itself serves as documentation.

---

## REQ-LANG-020 — Whitespace

Whitespace is not semantically significant except as a separator between tokens.
