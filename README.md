# Phenotyper

**Phenotyper** is a domain-specific language and compiler for defining the shape of structured textual artifacts and generating typed tooling that can construct, render, validate, and eventually parse them.

It is designed for outputs that are too structured to be treated as free-form text, but too artifact-shaped to fit naturally into ordinary data schemas alone.

Think:
- structured prompts
- CSV-like tabular text
- reports and semi-formal documents
- code and configuration fragments
- markup and transformation instructions
- other human-readable artifacts with a stable, meaningful shape

Phenotyper aims to make those outputs:
- more predictable
- more reusable
- easier to validate
- easier to generate safely from code
- easier to standardize across LLMs, tools, and time

---

## Why Phenotyper exists

Modern systems often need to produce artifacts that are not just data and not just prose.

A template engine can substitute values into a mostly textual skeleton. A schema language can define the shape of data. A parser generator can recognize input. But many real-world outputs live in the space between those tools.

Phenotyper is built for that middle space.

It lets you define an artifact family directly in a readable DSL, then generate typed Rust APIs that build valid instances of that family. Over time, the same definitions can also drive validation, parsing, and round-tripping.

In that sense, Phenotyper is closer to **"artifact schema + rendering algebra + generated tooling"** than to a plain template system.

---

## Core idea

A Phenotyper source file defines:
- a **namespace**
- reusable **types**
- named **phenotypes**
- singular/plural phenotype relationships
- typed fields
- render expressions that produce output
- a constrained structure for building valid artifacts

From that, the compiler can generate:
- typed builders
- renderers
- validators
- dedicated wrapper types for plural companion phenotypes
- Rust enums for named enum types and union-like field choices
- later, parsers and reverse mappings

---

## A first taste

### Markdown-based source form

Phenotyper supports documentation-rich source documents in Markdown. Phenotype code lives in fenced `pht` blocks.

````markdown
# CSV artifact family

This family models a CSV-like format.

```pht
aivolution/format/csv:

type ScalarValue: {int64, real64, string, date, time, datetime};
type Visibility: [public, protected, private];

CSVFieldValue plural CSVFieldValues:
    value: required ScalarValue,
    @(value)
;

CSVLine plural CSVLines:
    values: required CSVFieldValues,
    separator: required string,
    @join(values, separator)
;

.
```
````

### Pure `.pht` source form

```pht
aivolution/format/csv:

// Reusable union-like type
type ScalarValue: {int64, real64, string, date, time, datetime};

/* Closed symbolic enum type */
type Visibility: [public, protected, private];

CSVFieldValue plural CSVFieldValues:
    value: required ScalarValue,
    @(value)
;

.
```

---

## Language highlights

### Structural Namespaces

Phenotyper uses structural namespace declarations with `/` as the
path separator, terminated by `:` at the start and `.` at the end:

```pht
aivolution/format/csv:

// ... declarations ...

.
```

This maps naturally to generated Rust modules:

- DSL namespace: `aivolution/format/csv`
- Rust module path: `aivolution::format::csv`

### Reusable named types

```pht
type ScalarValue: {int64, real64, string, Date, Time, Datetime};
type CsvText: string;
```

Phenotyper supports reusable type declarations using:

```pht
type Name: TypeExpression;
```

### Enums

Enums are namespace-level named types:

```pht
type Visibility: [public, protected, private];
```

These map naturally to dedicated Rust enums.

### Singular/plural phenotype declarations

A phenotype can declare its plural companion explicitly:

```pht
CSVLine plural CSVLines:
    values: required CSVFieldValues,
    separator: required string,
    @join(values, separator)
;
```

This makes the DSL more natural to read and allows code generation to preserve semantic collection types rather than collapsing everything into anonymous vectors.

### Nested phenotype declarations

Phenotype bodies can contain other phenotype declarations for modeling
hierarchical structures:

```pht
JavaClass plural JavaClasses:
    name: required string,

    Constructor plural Constructors:
        argList: optional string,
        @(JavaClass/name), "(", @(argList)?, ")"
    ;,

    ctors: required Constructors,
    "class ", @(name), " { ... }"
;
```

Nested types reference parent fields with `@(Parent/field)` and are
flattened into independent Rust structs at compile time.

### Render expressions

Phenotyper's output side is expressed through a small, explicit render-expression system.

Supported forms include:

```pht
"literal text"            // verbatim output
@(field)                  // field emission
@(Parent/field)           // parent-scoped field reference
@(optional_field)?        // optional field shorthand
@join(values, ", ")       // join collection with separator
@eol                      // end of line
@ifset(field) { ... }     // conditional on optional field
@ifnotempty(field) { ... } // conditional on non-empty collection
```

### Comments in pure `.pht`

Pure source files support:

```pht
// line comments
/* block comments */
```

---

## Two source containers, one core language

Phenotyper supports two normative source containers:

### `.md`
Markdown source documents.

- processed whether or not they contain phenotype content
- `pht` fenced blocks are extracted in document order
- markdown outside `pht` blocks is documentation only

### `.pht`
Pure Phenotyper source.

- parsed directly as the core language
- useful for tests, generated sources, and code-centric workflows

Both forms compile to the same core language model.

A key design requirement is that diagnostics must point to the **exact original line and column in the author’s source file**, whether that file is Markdown or pure `.pht`.

---

## Why not just use a template engine?

Template engines are useful, and Phenotyper is not trying to deny that.

But template engines and Phenotyper optimize for different things.

A general-purpose template engine like Handlebars or Jinja is excellent when you want:
- editable templates
- highly dynamic rendering logic
- familiar loops, helpers, includes, and macros
- looser coupling between structure and data model

Phenotyper is for cases where the artifact family itself deserves a proper type system and generated tooling.

It gives you:
- explicit structural definitions
- type-checked construction APIs
- dedicated collection wrapper types
- reusable union and enum types
- constrained rendering semantics
- a path toward parsing and round-tripping

### Potential performance angle

There is also a likely performance advantage.

A generated Rust builder/renderer for Phenotyper can often render more directly than a general-purpose template runtime because it already knows:
- the exact field set
- the exact output order
- legal cardinalities
- enum and union structure
- how joins and literals compose

That means the generated code can behave much closer to ordinary specialized Rust string-building code, with less runtime lookup and less generic template machinery.

A template engine is not necessarily doing naive string search-and-replace on every render, especially if templates are compiled and cached. In real use, the comparison is more fairly:

- **generated domain-specific Rust rendering code**
- versus **compiled general-purpose template runtime**

Phenotyper should usually have the advantage for fixed, repeatedly-rendered artifact families, especially where output shape matters. The biggest gains are likely to come from:
- reduced dynamic lookup
- simpler looping and join logic
- compile-time knowledge of structure
- fewer runtime shape errors

So the claim is not "templates are slow." The claim is:

> for stable, typed artifact families, generated Rust renderers can be both more reliable and potentially more efficient than general-purpose template execution.

---

## Generated Rust model

Phenotyper is designed to generate Rust that feels explicit and domain-shaped rather than generic and anonymous.

### Example direction

If a phenotype declares:

```pht
CSVLine plural CSVLines:
    values: required CSVFieldValues,
    @join(values, @", "),
    @eol()
;
```

then code generation can produce:
- `CsvLine`
- `CsvLines`
- `CsvLineBuilder`
- `CsvLines` as a dedicated wrapper type around `Vec<CsvLine>`

Plural companion types are intended to generate dedicated wrapper types, while still making the underlying vector representation easy to access through ergonomic conversions and helpers.

That gives you both:
- semantic clarity in the generated API
- practical collection ergonomics in Rust

### Builder-pattern API sketch

For a CSV-like phenotype such as:

```pht
aivolution/format/csv:

type ScalarValue: {int64, real64, string, date, time, datetime};

CSVFieldValue plural CSVFieldValues:
    value: required ScalarValue,
    @(value)
;

CSVRecord plural CSVRecords:
    values: required CSVFieldValues,
    @join(values, @", ")
;

CSVLine plural CSVLines:
    record: required CSVRecord,
    @(record)
;

CSVFile plural CSVFiles:
    lines: required CSVLines,
    @join(lines, @eol())
;
```

Phenotyper could generate Rust along these lines:

```rust
use std::fmt::{self, Write};

#[derive(Debug, Clone)]
pub enum ScalarValue {
    Int64(i64),
    Real64(f64),
    String(String),
    Date(String),
    Time(String),
    Datetime(String),
}

impl ScalarValue {
    pub fn render(&self, out: &mut String) -> fmt::Result {
        match self {
            ScalarValue::Int64(v) => write!(out, "{v}"),
            ScalarValue::Real64(v) => write!(out, "{v}"),
            ScalarValue::String(v) => write!(out, "{v}"),
            ScalarValue::Date(v) => write!(out, "{v}"),
            ScalarValue::Time(v) => write!(out, "{v}"),
            ScalarValue::Datetime(v) => write!(out, "{v}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsvFieldValue {
    value: ScalarValue,
}

impl CsvFieldValue {
    pub fn builder() -> CsvFieldValueBuilder {
        CsvFieldValueBuilder { value: None }
    }

    pub fn render(&self, out: &mut String) -> fmt::Result {
        self.value.render(out)
    }
}

pub struct CsvFieldValueBuilder {
    value: Option<ScalarValue>,
}

impl CsvFieldValueBuilder {
    pub fn value_int64(mut self, value: i64) -> Self {
        self.value = Some(ScalarValue::Int64(value));
        self
    }

    pub fn value_real64(mut self, value: f64) -> Self {
        self.value = Some(ScalarValue::Real64(value));
        self
    }

    pub fn value_string<S: Into<String>>(mut self, value: S) -> Self {
        self.value = Some(ScalarValue::String(value.into()));
        self
    }

    pub fn value_date<S: Into<String>>(mut self, value: S) -> Self {
        self.value = Some(ScalarValue::Date(value.into()));
        self
    }

    pub fn value_time<S: Into<String>>(mut self, value: S) -> Self {
        self.value = Some(ScalarValue::Time(value.into()));
        self
    }

    pub fn value_datetime<S: Into<String>>(mut self, value: S) -> Self {
        self.value = Some(ScalarValue::Datetime(value.into()));
        self
    }

    pub fn build(self) -> Result<CsvFieldValue, BuildError> {
        Ok(CsvFieldValue {
            value: self.value.ok_or(BuildError::MissingField("value"))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CsvFieldValues {
    items: Vec<CsvFieldValue>,
}

impl CsvFieldValues {
    pub fn builder() -> CsvFieldValuesBuilder {
        CsvFieldValuesBuilder { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<CsvFieldValue>) -> Self {
        Self { items }
    }

    pub fn into_vec(self) -> Vec<CsvFieldValue> {
        self.items
    }

    pub fn render_joined(&self, out: &mut String, joiner: &str) -> fmt::Result {
        let mut first = true;
        for item in &self.items {
            if !first {
                out.push_str(joiner);
            }
            first = false;
            item.render(out)?;
        }
        Ok(())
    }
}

pub struct CsvFieldValuesBuilder {
    items: Vec<CsvFieldValue>,
}

impl CsvFieldValuesBuilder {
    pub fn push(mut self, value: CsvFieldValue) -> Self {
        self.items.push(value);
        self
    }

    pub fn push_string<S: Into<String>>(mut self, value: S) -> Self {
        let field = CsvFieldValue::builder()
            .value_string(value)
            .build()
            .expect("builder generated invalid field");
        self.items.push(field);
        self
    }

    pub fn push_int64(mut self, value: i64) -> Self {
        let field = CsvFieldValue::builder()
            .value_int64(value)
            .build()
            .expect("builder generated invalid field");
        self.items.push(field);
        self
    }

    pub fn build(self) -> Result<CsvFieldValues, BuildError> {
        Ok(CsvFieldValues { items: self.items })
    }
}

#[derive(Debug, Clone)]
pub struct CsvRecord {
    values: CsvFieldValues,
}

impl CsvRecord {
    pub fn builder() -> CsvRecordBuilder {
        CsvRecordBuilder { values: None }
    }

    pub fn render(&self, out: &mut String) -> fmt::Result {
        self.values.render_joined(out, ", ")
    }
}

pub struct CsvRecordBuilder {
    values: Option<CsvFieldValues>,
}

impl CsvRecordBuilder {
    pub fn values(mut self, values: CsvFieldValues) -> Self {
        self.values = Some(values);
        self
    }

    pub fn build(self) -> Result<CsvRecord, BuildError> {
        Ok(CsvRecord {
            values: self.values.ok_or(BuildError::MissingField("values"))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CsvLine {
    record: CsvRecord,
}

impl CsvLine {
    pub fn builder() -> CsvLineBuilder {
        CsvLineBuilder { record: None }
    }

    pub fn render(&self, out: &mut String) -> fmt::Result {
        self.record.render(out)
    }
}

pub struct CsvLineBuilder {
    record: Option<CsvRecord>,
}

impl CsvLineBuilder {
    pub fn record(mut self, record: CsvRecord) -> Self {
        self.record = Some(record);
        self
    }

    pub fn build(self) -> Result<CsvLine, BuildError> {
        Ok(CsvLine {
            record: self.record.ok_or(BuildError::MissingField("record"))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CsvLines {
    items: Vec<CsvLine>,
}

impl CsvLines {
    pub fn builder() -> CsvLinesBuilder {
        CsvLinesBuilder { items: Vec::new() }
    }

    pub fn render_joined(&self, out: &mut String, eol: &str) -> fmt::Result {
        let mut first = true;
        for item in &self.items {
            if !first {
                out.push_str(eol);
            }
            first = false;
            item.render(out)?;
        }
        Ok(())
    }
}

pub struct CsvLinesBuilder {
    items: Vec<CsvLine>,
}

impl CsvLinesBuilder {
    pub fn push(mut self, line: CsvLine) -> Self {
        self.items.push(line);
        self
    }

    pub fn build(self) -> Result<CsvLines, BuildError> {
        if self.items.is_empty() {
            return Err(BuildError::CardinalityViolation("lines must not be empty"));
        }
        Ok(CsvLines { items: self.items })
    }
}

#[derive(Debug, Clone)]
pub struct CsvFile {
    lines: CsvLines,
}

impl CsvFile {
    pub fn builder() -> CsvFileBuilder {
        CsvFileBuilder { lines: None }
    }

    pub fn render(&self) -> Result<String, fmt::Error> {
        let mut out = String::new();
        self.lines.render_joined(&mut out, "\n")?;
        Ok(out)
    }
}

pub struct CsvFileBuilder {
    lines: Option<CsvLines>,
}

impl CsvFileBuilder {
    pub fn lines(mut self, lines: CsvLines) -> Self {
        self.lines = Some(lines);
        self
    }

    pub fn build(self) -> Result<CsvFile, BuildError> {
        Ok(CsvFile {
            lines: self.lines.ok_or(BuildError::MissingField("lines"))?,
        })
    }
}

#[derive(Debug)]
pub enum BuildError {
    MissingField(&'static str),
    CardinalityViolation(&'static str),
}
```

And using that generated API could look like this:

```rust
let record1 = CsvRecord::builder()
    .values(
        CsvFieldValues::builder()
            .push_string("Alice")
            .push_int64(42)
            .build()?
    )
    .build()?;

let record2 = CsvRecord::builder()
    .values(
        CsvFieldValues::builder()
            .push_string("Bob")
            .push_int64(37)
            .build()?
    )
    .build()?;

let csv = CsvFile::builder()
    .lines(
        CsvLines::builder()
            .push(CsvLine::builder().record(record1).build()?)
            .push(CsvLine::builder().record(record2).build()?)
            .build()?
    )
    .build()?;

println!("{}", csv.render()?);
```

Output:

```text
Alice, 42
Bob, 37
```

That is the API style Phenotyper is aiming for: generated Rust that follows the builder pattern, preserves the vocabulary of the phenotype, and makes invalid output harder to construct.

---

## Design principles

Phenotyper is being shaped around a few core principles.

### Human-readable first
The DSL should remain readable to humans and preserve the visible structure of the artifact family.

### Declarative structure
Phenotyper should describe **what a valid artifact looks like**, not become a general-purpose programming language.

### Generated tooling from one source of truth
A single source definition should drive builders, renderers, validators, and later parsers.

### Strong artifact identity
Collections, enums, union-like field choices, and render expressions should remain visible as first-class concepts.

### Documentation-friendly authoring
Phenotypes should be easy to explain inline, which is why Markdown-based source documents are first-class.

---

## Planned compiler pipeline

A likely v1 compiler pipeline looks like this:

1. Read source container (`.md` or `.pht`)
2. If Markdown, extract `pht` blocks and build a source map
3. Parse the core Phenotyper language
4. Build an AST
5. Normalize to an IR
6. Resolve namespaces, `uses`, and type references
7. Validate structure, cardinality, and render-expression correctness
8. Generate Rust builders, types, and renderers
9. Later: generate parsers and reverse mappings

---

## Name resolution model

Phenotyper's name-resolution model is intentionally simple.

- every file belongs to exactly one namespace
- local names must be unique within that namespace
- `uses` brings all declarations from a namespace into scope in v1
- local declarations shadow imported ones, with a compiler warning
- ambiguous imported names are hard errors unless fully qualified
- duplicate names within the same namespace are hard errors

---

## Current status

Phenotyper v1 is a **working compiler** with a complete pipeline:

| Component | Status |
|-----------|--------|
| Lexer & source map | ✅ Tokenizer with markdown extraction |
| Parser | ✅ Rustemo-based PEG grammar |
| Symbol table | ✅ Two-pass name resolution |
| Intermediate representation | ✅ Normalized IR with validation |
| Semantic validation | ✅ Type, render, and generation checks |
| Diagnostics | ✅ Rich human-readable and JSON output |
| Code generation | ✅ Idiomatic Rust with rustfmt |
| CLI | ✅ `check`, `build`, `dump-ast`, `dump-ir` |
| Build integration | ✅ `phenotyper_core::compile()` API |
| Test suite | ✅ 219 tests (unit, e2e, CLI, compile, runtime) |

### Quick start

```bash
# Install from source
cargo install --path crates/phenotyper-cli

# Check a source file
phenotyper check path/to/file.pht

# Generate Rust code
phenotyper build path/to/file.pht --out generated/

# Use from build.rs
# See docs/howto/build_rs_integration.md
```

### Documentation

- [Authoring phenotypes](docs/howto/authoring_phenotypes.md) — DSL syntax guide
- [Using generated code](docs/howto/using_generated_code.md) — Rust API guide
- [Compiler usage](docs/howto/compiler_usage.md) — CLI reference
- [Build script integration](docs/howto/build_rs_integration.md) — `build.rs` guide
- [Worked examples](docs/examples/) — CSV, prompt, config, report

---

## Project goals

Phenotyper is a foundation for:
- structured prompt engineering
- robust artifact generation for AI systems
- typed textual interfaces
- reusable format definitions
- eventually, round-trippable artifact specifications

> Define the shape of an artifact once, then generate the tooling needed to create it correctly.

---

## License

Apache-2.0
