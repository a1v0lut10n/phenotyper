# REQ-CODEGEN — Rust Code Generation Requirements

## Status
v1 — Derived from `phenotyper_v1_language_and_compiler_spec.md` sections 19–20.

---

## REQ-CODEGEN-001 — Per-type output shape

For each phenotype type, code generation shall produce:

| Artifact | Description |
|----------|-------------|
| Singular struct | Rust `struct` representing one instance |
| Plural wrapper struct | Dedicated Rust wrapper type for the collection (when `plural` declared) |
| Builder struct | Type-safe builder for constructing instances |
| Render implementation | `Render` trait implementation or method |
| Validation methods | Runtime checks in `build()` |

---

## REQ-CODEGEN-002 — Singular struct generation

For a phenotype like:

```pht
CSVLine plural CSVLines:
    fields: required CSVFieldValues,
    separator: required string,
    @join(fields, separator)
;
```

Generate:

```rust
#[derive(Debug, Clone)]
pub struct CsvLine {
    pub fields: CsvFieldValues,
    pub separator: String,
}
```

- Required singular fields → plain Rust struct fields.
- Optional fields → `Option<T>`.
- Union fields → generated enum.

---

## REQ-CODEGEN-003 — Plural wrapper struct generation

When a phenotype declares a plural companion, generate a dedicated wrapper type:

```rust
#[derive(Debug, Clone)]
pub struct CsvLines {
    items: Vec<CsvLine>,  // private storage
}
```

Minimum required API on plural wrappers:

| Method | Signature |
|--------|-----------|
| `new` | `fn new() -> Self` |
| `from_vec` | `fn from_vec(items: Vec<T>) -> Self` |
| `into_vec` | `fn into_vec(self) -> Vec<T>` |
| `as_slice` | `fn as_slice(&self) -> &[T]` |
| `iter` | `fn iter(&self) -> std::slice::Iter<'_, T>` |
| `From<Vec<T>>` | Trait implementation |
| `Into<Vec<T>>` | Trait implementation (via `From`) |

Internal storage (`items`) shall be **private** in v1 to allow future invariant enforcement.

**Open decision:** Whether plural wrappers implement `Deref<[T]>` in v1, or only explicit collection methods.

---

## REQ-CODEGEN-004 — Builder API generation

### Pattern

- Empty builder via `TypeName::builder()` (preferred) or `TypeNameBuilder::new()`.
- Setters use `&mut self` and return `&mut Self` for chaining in v1.
- `build() -> Result<TypeName, BuildError>` performs runtime validation.

### Builder for singular types

```rust
pub struct CsvLineBuilder {
    fields: Option<CsvFieldValues>,
    separator: Option<String>,
}

impl CsvLineBuilder {
    pub fn fields(&mut self, val: CsvFieldValues) -> &mut Self { ... }
    pub fn separator(&mut self, val: String) -> &mut Self { ... }
    pub fn build(self) -> Result<CsvLine, BuildError> { ... }
}
```

### Builder for plural types

Plural builders support `push` and `extend` operations:

```rust
pub struct CsvLinesBuilder {
    items: Vec<CsvLine>,
}

impl CsvLinesBuilder {
    pub fn push(&mut self, item: CsvLine) -> &mut Self { ... }
    pub fn build(self) -> Result<CsvLines, BuildError> { ... }
}
```

Union-typed fields should generate per-variant push convenience methods on plural builders (e.g., `push_string`, `push_int64`).

---

## REQ-CODEGEN-005 — BuildError type

```rust
#[derive(Debug)]
pub enum BuildError {
    MissingField(&'static str),
    CardinalityViolation(&'static str),
}
```

---

## REQ-CODEGEN-006 — Render trait

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

- `render_into` is the core method that all generated types implement. It writes directly into the provided buffer for zero-allocation recursive rendering.
- `render` is a convenience wrapper with a default implementation.

All generated singular phenotype structs shall implement `Render`.

Plural companion structs shall implement rendering via their join semantics as referenced in the phenotype definition.

### Scalar rendering defaults

| Type | Rendering |
|------|-----------|
| `string` | Emitted as-is |
| `int64` | Decimal representation |
| `real64` | Rust default float format |
| `bool` | `true` / `false` |
| `date`, `time`, `datetime` | ISO-oriented canonical string form |

---

## REQ-CODEGEN-007 — Union type mapping

A field with type `{string, int64, MyType}` shall generate a Rust enum:

```rust
pub enum FieldNameValue {
    String(String),
    Int64(i64),
    MyType(MyType),
}
```

- Enum naming follows deterministic rules based on the containing type and field names.
- Each variant implements rendering by delegating to the contained type.

---

## REQ-CODEGEN-008 — Enum type mapping

A namespace-level enum:

```pht
type Visibility: [public, protected, private];
```

Generates:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}
```

- Variant names are derived deterministically (PascalCase).
- Default rendering emits the **original declared spelling** (e.g., `"public"`, not `"Public"`).

---

## REQ-CODEGEN-009 — Naming strategy

| DSL declaration | Rust output |
|-----------------|-------------|
| `CSVLine` | `CsvLine` |
| `CSVLines` (plural) | `CsvLines` |
| builder for `CSVLine` | `CsvLineBuilder` |
| builder for `CSVLines` | `CsvLinesBuilder` |

Type names are converted to idiomatic Rust `PascalCase` while preserving the vocabulary of the phenotype.

---

## REQ-CODEGEN-010 — Validation placement

Two validation layers:

| Layer | Timing | Checks |
|-------|--------|--------|
| Compile-time | During compilation | Language definition errors (duplicate names, invalid directives, type errors) |
| Runtime | At `build()` call | Missing required fields, empty `OneOrMore` collections |

---

## REQ-CODEGEN-011 — Escaping policy

Phenotyper v1 does **not** impose automatic escaping rules. Values are rendered literally.

Escaping-sensitive formats are the responsibility of the phenotype design or later directive extensions.

---

## REQ-CODEGEN-012 — Module structure

Generated Rust code shall be organized into modules matching the namespace path:

- `aivolution/format/csv` → `aivolution/format/csv.rs` (or `aivolution/format/csv/mod.rs`)
- A root `mod.rs` shall re-export generated types.
