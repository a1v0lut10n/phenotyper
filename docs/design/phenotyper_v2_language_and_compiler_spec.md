# Phenotyper v2 — Language and Compiler Specification

## Status

Draft — design document for the next major language revision.

This spec describes three language enhancements proposed for v2.
Each is motivated by the javaclass.md example and by real authoring
experience with the v1 language.

---

## Overview of Changes

| # | Feature | v1 | v2 |
|---|---------|----|----|
| 1 | Namespace syntax | `namespace a/b/c;` (keyword) | `a/b/c:` ... `.` (structural) |
| 2 | Nested phenotypes | Not supported | Nested with parent-scoped field access |
| 3 | `?` suffix operator | Not supported | `field?` shorthand for `@ifset`/`@ifnotempty` |

---

## 1. Structural Namespace Syntax

### 1.1 Motivation

In v1, namespaces use a dedicated keyword:

```pht
namespace aivolution/format/csv;
```

This makes namespaces a *statement*, not a *structural form*. But
namespaces are conceptually containers — they scope the declarations
that follow. The javaclass.md example treats the namespace as a
structural definition terminated by `.`:

```pht
java/generation:
```

This is consistent with how phenotypes use `:` to begin their body.
Namespaces are to files what phenotypes are to instances.

### 1.2 Proposed Syntax

A namespace is declared by a namespace path followed by `:`,
and terminated by `.` at the end of the file:

```pht
aivolution/format/csv:

type ScalarValue: {int64, real64, string};

CsvFieldValue plural CsvFieldValues:
    value: required ScalarValue,
    @(value)
;

.
```

### 1.3 Grammar Changes

v1:
```
File: ns=NamespaceDecl uses=UsesDecl* decls=TopLevelDecl*;
NamespaceDecl: 'namespace' path=NamespacePath ';';
```

v2:
```
File: ns=NamespaceScope | decls=TopLevelDecl+;
NamespaceScope: path=NamespacePath ':' uses=UsesDecl* decls=TopLevelDecl* '.';
```

The `namespace` keyword is removed. A `NamespaceScope` opens with a
path and `:`, and closes with `.`. This makes the file a structural
scope just like a phenotype body, with a different terminator
(`.` vs `;`).

The `File` production is intentionally ambiguous at the `Ident ':'`
position — Rustemo's GLR mode resolves this structurally (see §1.4).

### 1.4 Disambiguation via GLR Parsing

A key question is how the parser distinguishes:

```pht
csv:                         ← namespace declaration? or phenotype?
aivolution/format/csv:       ← namespace declaration
CsvFieldValue:               ← phenotype declaration
```

The forms `Name:` and `path/name:` are syntactically identical at
the opening token. Rustemo supports **GLR (Generalized LR) parsing**,
which resolves this ambiguity structurally by forking the parse:

1. The parser encounters `Ident ':'` and forks into two branches:
   - **Branch A (namespace):** Expects `uses`, `type`, or phenotype
     declarations, terminated by `.`
   - **Branch B (phenotype):** Expects field declarations and render
     expressions, terminated by `;`

2. The branches diverge quickly because their body grammars are
   fundamentally different:

   | Construct | Body expects | Terminator |
   |-----------|-------------|------------|
   | Namespace | `UsesDecl`, `TypeDecl`, `TypeDef` | `.` |
   | Phenotype | `FieldDecl`, `RenderExpr`, nested `TypeDef` | `;` |

3. When one branch encounters a token incompatible with its
   grammar, Rustemo prunes it. The surviving branch is the
   correct parse.

**Why GLR is the right strategy:**

- **No artificial restrictions.** Single-segment namespaces like
  `csv:` are valid — no requirement for `/` or multiple segments.
- **No mandatory keywords.** `plural` remains optional for
  phenotypes; it is not needed for disambiguation.
- **Clean syntax.** The author writes natural paths and names
  without worrying about parser limitations.
- **Leverages existing tooling.** Rustemo's GLR engine is
  battle-tested and handles this class of ambiguity efficiently.

**Worst-case ambiguity:** An empty namespace `csv: .` and an empty
phenotype `csv: ;` are both minimal. The terminator (`.` vs `;`)
resolves this unambiguously — no phenotype can end with `.`, and
no namespace can end with `;`.

**Multi-segment paths:** Namespaces with `/` in the path (e.g.,
`aivolution/format/csv:`) are trivially unambiguous even without
GLR, since phenotype names are single identifiers. GLR is only
needed for the single-segment edge case.

### 1.5 Termination: `.` vs `;`

The namespace scope terminates with `.` rather than `;`. This creates
a visible *end-of-file* marker and differentiates the file-level
scope from phenotype definitions:

| Scope | Opens with | Terminates with |
|-------|------------|-----------------|
| File/namespace | `path:` | `.` |
| Phenotype | `Name:` | `;` |
| Nested phenotype | `Name:` | `;` |

The `.` serves as an explicit EOF assertion. It is optional at the
end of the file if we want backwards-compatible parsing, but
**recommended** for clarity.

### 1.6 Migration from v1

| v1 | v2 |
|----|----|
| `namespace aivolution/format/csv;` | `aivolution/format/csv:` |
| (implicit end-of-file) | `.` at end of file |

A mechanical migration tool can convert v1 files to v2 by:
1. Removing the `namespace` keyword and trailing `;`
2. Adding `.` at the end of the file

### 1.7 Impact on `.md` Containers

In `.md` files, the namespace declaration appears in the first
`pht` block. The `.` terminator is **implicit** — the end of the
extracted blocks serves as the namespace scope terminator:

````markdown
# CSV Format

```pht
aivolution/format/csv:
```

## Fields

```pht
CsvFieldValue:
    value: required string,
    @(value)
;
```
````

No trailing ` ```pht . ``` ` block is needed. The markdown
extraction logic concatenates all `pht` blocks in document order
and appends an implicit `.` at the end. This keeps markdown sources
clean and avoids an awkward code block containing only a period.

In `.pht` files, the explicit `.` terminator remains **required**.

| Container | `.` terminator |
|-----------|----------------|
| `.pht` | Required (explicit) |
| `.md` | Implicit (end of extracted blocks) |

---

## 2. Nested Phenotype Declarations

### 2.1 Motivation

Some artifact families naturally contain nested structure. The
javaclass.md example shows a `Constructor` phenotype nested inside
`JavaClass`:

```pht
JavaClass:
    name: key string,
    ...

    Constructor plural Constructors:
        visibility: required Visibility,
        argument plural arguments: optional Arguments,

        @(visibility), @space, @(JavaClass/name), "(", ...
    ;
;
```

The `Constructor` is logically part of `JavaClass` — it references
the parent `JavaClass/name` in its render body. In v1, all phenotypes
must be declared at namespace scope, forcing artificial flattening.

### 2.2 Proposed Syntax

A phenotype body can contain other phenotype declarations in
addition to fields and render expressions:

```pht
JavaClass plural JavaClasses:
    name: key string,
    interface plural interfaces: optional Interfaces,
    superClass: optional JavaClass,

    Constructor plural Constructors:
        visibility: required Visibility,
        argument plural arguments: optional Arguments,

        @(visibility), " ", @(JavaClass/name), "(", ...
    ;

    "public class ", @(name), " ", ...
;
```

### 2.3 Scoping Rules

Nested phenotypes introduce a **scope chain**. When resolving
field references, the compiler searches:

1. The current phenotype's own fields
2. The parent phenotype's fields (one level up)
3. Grandparent, etc. (up the nesting chain)
4. Namespace-level type declarations

Field references to parent fields use a **parent-qualified path**:

```pht
@(JavaClass/name)       // References the 'name' field of the containing JavaClass
@(name)                 // References own field (or nearest in scope)
```

The `/` separator serves double duty:
- In namespace paths: `aivolution/format/csv` (segment separator)
- In scoped field refs: `JavaClass/name` (scope resolution)

This is consistent — both are hierarchical path expressions.

### 2.4 Scoped Field Reference Resolution

The field path `@(A/B)` resolves as follows:

1. **Named scope:** If `A` matches a containing phenotype name,
   resolve `B` in that phenotype's field declarations.
2. **Field subpath:** If `A` matches a field name in the current
   scope, resolve `B` as a sub-field of `A`'s type (future).
3. **Error:** If `A` matches neither, emit a diagnostic.

For v2, **only (1) is supported** — scoped references to containing
phenotype fields. Sub-field paths like `record/values` (from the
javaclass example's `methodSignature/argument/type`) are deferred
to v3.

### 2.5 Grammar Changes

v1:
```
BodyItem: field=FieldDecl {Field}
        | render=RenderExpr {Render}
        ;
```

v2:
```
BodyItem: field=FieldDecl {Field}
        | render=RenderExpr {Render}
        | nested=TypeDef {NestedType}
        ;
```

`TypeDef` is already defined in v1. Making it a valid `BodyItem`
enables nesting without new grammar rules. The `TypeDef` production
is recursive by definition.

### 2.6 Nested Field References in Render Expressions

The `FieldRef` render expression is extended to accept a path:

v1:
```
RenderExpr: '@' '(' ref_name=Ident ')' {FieldRef};
```

v2:
```
RenderExpr: '@' '(' ref_path=FieldPath ')' {FieldRef};
FieldPath: segments=Ident+[Slash];
```

This allows both:
- `@(name)` — simple field reference (one segment)
- `@(JavaClass/name)` — scoped reference (two segments)

### 2.7 Code Generation for Nested Phenotypes

Nested phenotypes generate **flat Rust types** at the module level.
The nesting is a source-level scoping mechanism only; it does not
create nested Rust modules or types.

```rust
// Source: Constructor nested inside JavaClass
// Generated: both at module level
pub struct JavaClass { ... }
pub struct Constructor { ... }
```

The parent reference `@(JavaClass/name)` generates a render method
that takes the parent as a parameter, or captures it at construction
time. Two strategies:

#### Strategy A: Render-time parent parameter

```rust
impl Constructor {
    pub fn render_with_parent(&self, parent: &JavaClass, out: &mut String) {
        out.push_str(&self.visibility.render());
        out.push(' ');
        out.push_str(&parent.name);  // @(JavaClass/name)
        // ...
    }
}
```

#### Strategy B: Constructor captures parent context

```rust
pub struct Constructor {
    pub visibility: Visibility,
    pub arguments: Arguments,
    pub parent_name: String,  // Captured from JavaClass/name
}
```

> [!IMPORTANT]
> **Recommended: Strategy A** — render-time parent parameter. This
> avoids data duplication and keeps the generated types independent.
> The trade-off is a slightly more complex render API for nested
> types. Strategy B is simpler but creates coupling and potential
> inconsistency if the parent changes.

### 2.8 Name Visibility

Nested phenotypes are **visible only within their parent**. A
`Constructor` declared inside `JavaClass` cannot be referenced
from a sibling `Interface` phenotype unless explicitly exported.

In v2, nested types are **not exportable** — they are private to
the containing phenotype. This keeps the scoping rules simple and
avoids introducing an access-control system.

At the Rust level, all generated types are still `pub` (flat module).
The visibility restriction is enforced in the DSL, not in the
generated code.

### 2.9 Nesting Depth Limits

v2 allows **arbitrary nesting depth** but recommends at most **two
levels** (parent → child). The compiler should emit a warning for
nesting deeper than 3 levels.

---

## 3. The `?` Suffix Operator

### 3.1 Motivation

v1's `@ifset(field) { ... }` and `@ifnotempty(field) { ... }` are
verbose for the common case of conditionally emitting a field:

```pht
// v1: verbose
@ifset(subtitle) { "## ", @(subtitle), "\n" }
@ifnotempty(tags) { "\n---\nTags: ", @join(tags, ", ") }
```

The javaclass.md example uses a suffix `?` for the same purpose:

```pht
methodSignatures?
    foreach(methodSignature):
        ...
```

### 3.2 Proposed Syntax

The `?` suffix operator attaches to a field reference and makes
the subsequent render expressions conditional:

```pht
// v2: concise
@(subtitle)? { "## ", @(subtitle), "\n" }
@(tags)? { "\n---\nTags: ", @join(tags, ", ") }
```

Or, for single-expression emission:

```pht
@(subtitle)?       // emits the field if Some, else nothing
```

### 3.3 Semantics

The `?` operator is polymorphic based on field type:

| Field Type | `?` Behavior | Equivalent v1 |
|------------|-------------|---------------|
| `optional T` | Emit only if `Some` | `@ifset(field)` |
| Collection (`*`) | Emit only if non-empty | `@ifnotempty(field)` |
| `required T` | Compile-time warning | Always emits |

### 3.4 Bare vs Block Form

#### Bare form: `@(field)?`

When `?` appears without a block, it emits the field value directly
if the condition is met:

```pht
@(subtitle)?           // If Some("Draft"), emits "Draft"
                       // If None, emits nothing
```

This is equivalent to:

```pht
@ifset(subtitle) { @(subtitle) }
```

#### Block form: `@(field)? { ... }`

When `?` is followed by a block, the entire block is conditional:

```pht
@(subtitle)? { "## ", @(subtitle), "\n" }
```

Inside the block, `@(field)` refers to the **unwrapped** value
(same as `@ifset` semantics).

### 3.5 Grammar Changes

v1:
```
RenderExpr: '@' '(' ref_name=Ident ')' {FieldRef};
```

v2 (additional production):
```
RenderExpr: '@' '(' ref_path=FieldPath ')' '?' block=BlockBody? {ConditionalRef};
```

### 3.6 Interaction with Nested Phenotype Scope

The `?` operator works with scoped references:

```pht
@(JavaClass/superClass)? { " extends ", @(JavaClass/superClass/name) }
```

### 3.7 Compatibility with `@ifset` / `@ifnotempty`

The `?` operator is syntactic sugar. `@ifset` and `@ifnotempty`
remain valid in v2. The two forms are interchangeable:

```pht
// These are equivalent:
@(subtitle)? { "## ", @(subtitle) }
@ifset(subtitle) { "## ", @(subtitle) }

// These are equivalent:
@(tags)? { @join(tags, ", ") }
@ifnotempty(tags) { @join(tags, ", ") }
```

The compiler normalizes `?` to the corresponding `@ifset` or
`@ifnotempty` IR node during lowering.

### 3.8 Edge Cases

**`?` on a required field:**

```pht
name: required string,
@(name)?               // Warning: field is always present
```

The compiler emits a diagnostic: *"'?' on required field 'name' is
redundant — the field always has a value."* The code generates
correctly (always emits), but the `?` is misleading.

**`?` on a collection with `+` cardinality:**

```pht
items: required Tags+,
@(items)?              // Warning: one-or-more is never empty
```

Same treatment — warning, but generates correctly.

---

## 4. Cross-Cutting Concerns

### 4.1 Grammar Summary

v2 grammar delta from v1:

```diff
-File: ns=NamespaceDecl uses=UsesDecl* decls=TopLevelDecl*;
-NamespaceDecl: 'namespace' path=NamespacePath ';';
+File: ns=NamespaceScope | decls=TopLevelDecl+;
+NamespaceScope: path=NamespacePath ':' uses=UsesDecl* decls=TopLevelDecl* '.';

 BodyItem: field=FieldDecl {Field}
         | render=RenderExpr {Render}
+        | nested=TypeDef {NestedType}
         ;

-RenderExpr: '@' '(' ref_name=Ident ')' {FieldRef}
+FieldPath: segments=Ident+[Slash];
+RenderExpr: '@' '(' ref_path=FieldPath ')' {FieldRef}
+          | '@' '(' ref_path=FieldPath ')' '?' block=BlockBody? {ConditionalRef}
```

> [!NOTE]
> The `File` production is intentionally ambiguous at `Ident ':'`.
> Rustemo's GLR mode resolves this by forking and pruning based on
> the body grammar and terminator (`.` vs `;`). See §1.4.

### 4.2 Reserved Words

v2 removes `namespace` from the reserved word list. No new
keywords are introduced — `?` and `.` are punctuation tokens.

### 4.3 Source Compatibility

v1 sources **are not compatible** with the v2 parser due to the
namespace syntax change. A migration tool (`phenotyper migrate`)
should be provided to automate the conversion.

### 4.4 Error Recovery

The `.` terminator at end-of-file helps the parser detect
unterminated scopes. If the parser reaches EOF without `.`, it
can emit: *"expected '.' to end namespace scope"*.

### 4.5 Diagnostic Improvements

Nested phenotype scoping requires enhanced diagnostics:

- *"field 'foo' not found in scope; did you mean 'Parent/foo'?"*
- *"nested phenotype 'Child' references field 'bar' from parent
  'Parent', which is optional — use '?' or @ifset"*

---

## 5. Worked Example: v2 CSV

```pht
aivolution/format/csv:

type ScalarValue: {int64, real64, string, date, time, datetime};

CsvFieldValue plural CsvFieldValues:
    value: required ScalarValue,
    @(value)
;

CsvLine plural CsvLines:
    fields: required CsvFieldValues,
    separator: required string,
    @join(fields, separator)
;

CsvFile:
    header: required CsvLine,
    lines: required CsvLines,
    @(header), "\n",
    @join(lines, "\n")
;

.
```

## 6. Worked Example: v2 Java Class (from javaclass.md)

```pht
java/generation:

type Visibility: [public, protected, private];

Argument plural Arguments:
    type: required string,
    name: required string,
    @(type), " ", @(name)
;

MethodSignature plural MethodSignatures:
    methodName: required string,
    visibility: required Visibility,
    returnType: required string,
    arguments: required Arguments,
    @(visibility), " ", @(returnType), " ", @(methodName),
    "(", @join(arguments, ", "), ")"
;

Interface plural Interfaces:
    name: required string,
    superInterfaces: required Interfaces*,
    methodSignatures: required MethodSignatures,
    "public interface ", @(name),
    @(superInterfaces)? { " extends ", @join(superInterfaces, ", ") },
    " {\n",
    @join(methodSignatures, "\n"), "\n",
    "}"
;

JavaClass plural JavaClasses:
    name: required string,
    interfaces: optional Interfaces,
    superClass: optional JavaClass,
    methodSignatures: optional MethodSignatures,

    Constructor plural Constructors:
        visibility: required Visibility,
        arguments: required Arguments*,
        @(visibility), " ", @(JavaClass/name), "(",
        @join(arguments, ", "),
        ") {\n}"
    ;

    "public class ", @(name),
    @(superClass)? { " extends ", @(superClass) },
    @(interfaces)? { " implements ", @join(interfaces, ", ") },
    " {\n",
    @(methodSignatures)? { @join(methodSignatures, "\n") },
    "}"
;

.
```

---

## 7. Implementation Roadmap

| Phase | Description | Effort |
|-------|-------------|--------|
| P1 | Namespace syntax change (grammar, parser, migration tool) | 2–3 days |
| P2 | `?` suffix operator (grammar, IR lowering, codegen) | 2–3 days |
| P3 | Nested phenotypes (grammar, scoping, codegen) | 5–7 days |
| P4 | Update documentation and examples | 2–3 days |
| P5 | Migration tool and backward compatibility testing | 1–2 days |

**Total estimated:** 12–18 working days.

### Suggested Order

1. **P1** first — namespace change is fundamental and affects every file
2. **P2** next — `?` is self-contained and low-risk
3. **P3** last — nested phenotypes are the most complex and can be
   incrementally developed

---

## 8. Open Questions

> [!NOTE]
> ### Q1: Single-segment namespaces ✅ RESOLVED
> Single-segment namespaces like `csv:` are valid. Disambiguation
> is handled by Rustemo's GLR parser, which forks on `Ident ':'`
> and prunes based on the body grammar and terminator (`.` vs `;`).
> No restriction on namespace path depth is imposed. See §1.4.

> [!NOTE]
> ### Q2: Trailing `.` in `.md` containers ✅ RESOLVED
> The `.` terminator is implicit in `.md` containers — the end of
> extracted `pht` blocks serves as the namespace scope terminator.
> In `.pht` files, the explicit `.` remains required. See §1.7.

> [!WARNING]
> ### Q3: Nested phenotype codegen strategy
> Which strategy for parent-scoped field references in generated
> code: render-time parameter, or captured context? This affects
> the public API surface of generated types.

> [!NOTE]
> ### Q4: Nested type declarations
> Should `type` declarations (enums, unions, aliases) also be
> nestable inside phenotype bodies, or only phenotype (`TypeDef`)
> declarations?

> [!NOTE]
> ### Q5: `?` on `@join`
> Should `@join(field, sep)?` be valid sugar for
> `@ifnotempty(field) { @join(field, sep) }`?
