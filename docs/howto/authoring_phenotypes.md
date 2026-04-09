# Authoring Phenotype Definitions

This guide explains how to write phenotype definitions in the Phenotyper
DSL, covering syntax, types, fields, render expressions, and best practices.

## Source Containers

Phenotyper supports two source formats:

### `.pht` — Pure Phenotyper Source

Parsed directly as the core language. Best for machine-generated sources
and code-centric workflows.

```pht
aivolution/format/csv:

type ScalarValue: {int64, real64, string, date, time, datetime};

CSVFieldValue plural CSVFieldValues:
    value: required ScalarValue,
    @(value)
;

.
```

### `.md` — Markdown with Embedded Phenotyper

Documentation-rich source documents where phenotype code lives in fenced
` ```pht ` blocks. The compiler extracts and concatenates all `pht`
blocks in document order.

````markdown
# My Format

This describes the format.

```pht
my/format:
```

## Records

```pht
Record:
    name: required string,
    @(name)
;

.
```
````

Both formats compile to the same language model and produce identical
output. Diagnostics always point to exact line/column in the original
source file.

## Namespace Declaration

Every phenotyper file must begin with a structural namespace declaration
followed by a colon. The file must end with a `.` terminator:

```pht
aivolution/format/csv:

// ... type and phenotype declarations ...

.
```

The namespace determines the output directory structure when code is
generated (e.g., `aivolution/format/csv/mod.rs`).

> **Note:** The v1 `namespace` keyword syntax is no longer supported.
> Use the structural `path/to/namespace:` form instead.

## Type Declarations

### Primitive Types

Phenotyper has the following primitive types:

| Type | Rust Type | Description |
|------|-----------|-------------|
| `string` | `String` | Text value |
| `int64` | `i64` | 64-bit signed integer |
| `real64` | `f64` | 64-bit floating point |
| `bool` | `bool` | Boolean value |
| `date` | `String` | Date (stored as string) |
| `time` | `String` | Time (stored as string) |
| `datetime` | `String` | Date+time (stored as string) |

### Union Types

A union type declares that a field can hold one of several types:

```pht
type ScalarValue: {int64, real64, string, date, time, datetime};
```

This generates a Rust enum:

```rust
pub enum ScalarValue {
    Int64(i64),
    Real64(f64),
    String(String),
    Date(String),
    Time(String),
    DateTime(String),
}
```

### Enum Types

An enum type declares a closed set of symbolic values:

```pht
type Visibility: [public, private, internal];
```

This generates a Rust enum with original-spelling rendering:

```rust
pub enum Visibility {
    Public,
    Private,
    Internal,
}
```

When rendered, the **original DSL spelling** is used (e.g., `"public"`,
not `"Public"`).

## Phenotype Declarations

### Basic Phenotype

A phenotype declares a named type with fields and render expressions:

```pht
Record:
    name: required string,
    value: required int64,
    @(name), ": ", @(value)
;
```

### Singular/Plural

A phenotype can declare a plural companion:

```pht
Record plural Records:
    name: required string,
    @(name)
;
```

The plural companion (`Records`) generates a wrapper struct around
`Vec<Record>` with collection helpers (`new()`, `from_vec()`,
`push()`, `iter()`, `len()`, `as_slice()`).

### Nested Phenotype Declarations

A phenotype body can contain other phenotype declarations in addition
to fields and render expressions. This is useful for modeling hierarchical
structures where a child type is logically part of a parent.

```pht
JavaClass plural JavaClasses:
    name: required string,

    Constructor plural Constructors:
        argList: optional string,
        @(JavaClass/name), "(", @(argList)?, ")"
    ;,

    ctors: required Constructors,

    "class ", @(name), " {", @eol,
    @join(ctors, "\n"),
    "}"
;
```

Key points:
- Nested types are terminated with `;,` (semicolon-comma) inside the
  parent body
- Nested types are flattened into independent Rust structs at compile time
- Nested types can reference parent fields using scoped paths (see below)
- Nested types can declare plurals like any top-level phenotype

## Fields

### Requiredness

```pht
name: required string,   // Must be set — generates String
label: optional string,  // May be None — generates Option<String>
```

### Cardinality

```pht
tags: required Tags*,    // Zero-or-more — always present, may be empty
```

### Field Types

Fields can reference:
- Primitive types: `string`, `int64`, `real64`, `bool`, `date`, `time`, `datetime`
- Union types: `ScalarValue` (a declared union)
- Enum types: `Visibility` (a declared enum)
- Other phenotypes: `Record` (singular) or `Records` (plural)

## Render Expressions

Render expressions define how a phenotype instance is transformed to text.

### Field Emission

```pht
@(field_name)
```

Emits the rendered value of the field.

### Parent-Scoped Field References

When inside a nested phenotype, you can reference fields from the parent
type using a scoped path:

```pht
@(ParentType/field_name)
```

For example, `@(JavaClass/name)` inside a `Constructor` accesses the
`name` field of the containing `JavaClass`. This generates a
`render_with_parent` method instead of the standard `render_into`.

### Optional Field Shorthand (`?` suffix)

The `?` operator provides a concise way to conditionally emit an optional
field:

```pht
@(optional_field)?
```

This is equivalent to:

```pht
@ifset(optional_field) { @(optional_field) }
```

The field renders only when its value is `Some(...)`.

### Text Literals

```pht
"hello, ", @(name), "!\n"
```

String literals are emitted verbatim. Standard escape sequences
(`\n`, `\t`, `\"`, `\\`) are supported.

### Join

```pht
@join(collection_field, separator)
```

Joins the items in a collection with a separator:

```pht
@join(items, ", ")         // literal separator
@join(items, separator)    // field-based separator
```

### End of Line

```pht
@eol                        // platform default EOL
@eol(eol_field)            // field-based EOL
```

### Conditional Rendering

#### `@ifset` — Optional Fields

```pht
@ifset(optional_field) { "prefix: ", @(optional_field) }
```

The block renders only when the optional field is `Some(...)`.

#### `@ifnotempty` — Collection Fields

```pht
@ifnotempty(collection_field) { "Items: ", @join(collection_field, ", ") }
```

The block renders only when the collection is non-empty.

## Comments

In `.pht` files:

```pht
// Line comment
/* Block comment */
```

In `.md` files, all text outside ` ```pht ` blocks is documentation
and is ignored by the compiler.

## Best Practices

1. **Use descriptive names**: `CSVFieldValue` over `Field`.
2. **Declare plurals explicitly**: `Record plural Records:` makes the
   generated API more expressive.
3. **Use union types for variant fields**: Rather than `string`, define
   `type ScalarValue: {int64, string};` for type safety.
4. **Mark fields `required` or `optional`**: Explicit requiredness makes
   the generated builder validation predictable.
5. **Use `.md` for documented libraries**: The markdown format lets you
   embed design rationale alongside the definitions.
6. **Use `.pht` for generated sources**: Pure files are simpler for
   machine-to-machine workflows.
7. **Use nesting for logically contained types**: If a child type only
   makes sense in the context of its parent, declare it nested.
8. **Use `?` for simple optional emission**: `@(field)?` is more readable
   than `@ifset(field) { @(field) }` for single-field conditionals.
