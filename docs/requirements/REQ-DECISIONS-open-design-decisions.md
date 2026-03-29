# REQ-DECISIONS — Open Design Decisions for v1

## Status
v1 — Consolidated from `phenotyper_v1_language_and_compiler_spec.md` sections 26–27 and `phenotyper_design_memo.md`.

---

These decisions should be resolved early in implementation. For each item, the **recommended v1 choice** from the spec is listed.

---

## DEC-001 — Optional field support depth ✅ RESOLVED

**Question:** Is `optional` fully enabled in v1, or merely parsed and partially validated?

**Decision:** `optional` is fully enabled in v1. Two companion directives handle conditional rendering:

- `@ifset(field) { render_body }` — renders the body only when the optional field has a value. Within the body, `@(field)` emits the unwrapped value.
- `@ifnotempty(field) { render_body }` — renders the body only when a collection field is non-empty. Pairs naturally with `@join` inside the body.

Direct `@(field)` interpolation of optional fields **outside** an `@ifset` block remains a compile-time error, preserving safety. This avoids general-purpose `if/else` while giving optional and collection fields clean declarative handling.

---

## DEC-002 — `@eol` directive status ✅ RESOLVED

**Question:** Does `@eol` survive as a distinct directive or lower to `@emit`?

**Decision:** `@eol` is a **distinct directive** in v1. It is the newline-emitting mechanism:

- `@eol()` — emits a newline character (`\n`).
- `@eol(field)` — emits the field value as a scalar string, **followed by** a newline.

This is intentionally different from `@emit`, which never emits trailing newlines. `@eol` gives explicit, visible control over line boundaries in the rendered output.

---

## DEC-003 — Zero-or-more (`*`) repetition ✅ RESOLVED

**Question:** Is `*` repetition accepted in v1?

**Decision:** Yes. Both `+` (one-or-more) and `*` (zero-or-more) are fully enabled in v1. Grammar changes are expensive to retrofit, and having both cardinality operators from the start keeps the language consistent. `*` pairs naturally with `@ifnotempty` for conditional rendering of potentially empty collections.

---

## DEC-004 — Builder setter style ✅ RESOLVED

**Question:** Should builder setters consume `self` or use `&mut self`?

**Decision:** Setters use `&mut self` returning `&mut Self` for chaining. The `build()` method consumes `self`. This allows fluent builder chains while ensuring the builder is invalidated after `build()`.

---

## DEC-005 — Temporal type backing ✅ RESOLVED

**Question:** Which Rust types back `date`, `time`, and `datetime`?

**Decision:** `String` backing in v1. The language syntax already reserves `date`, `time`, and `datetime` as distinct primitive types, but there is little value in separate Rust types without parsing and validation support in the generated builder API. v1 treats all three as `String` at the Rust level.

**Future enhancement:** Introduce proper typed backing (e.g., `chrono` or `time` crate types) with builder-level parsing/validation in a later version. The language syntax is already set up for this transition — only the codegen layer needs to change.

---

## DEC-006 — Recursive type graphs ✅ RESOLVED

**Question:** Are recursive type graphs rejected outright in the first release?

**Decision:** Yes. Reject cyclic type graphs in v1. This significantly simplifies ownership, rendering, and the generated Rust code.

---

## DEC-007 — Primitive type name resolution ✅ RESOLVED

**Question:** Are primitive type names fully reserved lexically, or resolved semantically?

**Decision:** Reserved lexically. `string`, `int64`, `real64`, `bool`, `date`, `time`, and `datetime` are reserved tokens in the lexer. They cannot be used as user-defined type or field names. This makes parsing unambiguous and errors clearer at the cost of a slightly larger reserved-word set.

---

## DEC-008 — Plural wrapper `Deref<[T]>` ✅ RESOLVED

**Question:** Do plural wrapper types implement `Deref<[T]>` in v1, or only expose explicit collection methods and conversions?

**Decision:** Explicit methods only (`from_vec`, `into_vec`, `as_slice`, `iter`, `From`/`Into` traits). No `Deref` in v1 to prevent method-resolution surprises and keep the API surface predictable. Evaluate `Deref` for v2 based on real user feedback.

---

## DEC-009 — Syntax order: field requiredness ✅ RESOLVED

**Observation:** The early example files (`csv.md`, `csv.pht`) use `field_name: required Type` syntax, while the v1 spec uses `required field_name: Type` syntax.

**Decision:** The **name-first** syntax is normative: `field_name: required TypeExpr`. The field name is the first thing a reader sees, making definitions more scannable. The v1 spec and all requirements documents have been updated to match this decision.

---

## DEC-010 — Namespace leading slash ✅ RESOLVED

**Observation:** The README and spec use `/aivolution/format/csv` (leading `/`), while the early examples use `aivolution/format/csv` (no leading `/`).

**Decision:** **No leading slash**. Namespace paths are written as `aivolution/format/csv`. This is cleaner and matches how namespaces are typically expressed. The v1 spec and all requirements documents have been updated to match this decision.

---

## DEC-011 — `@eol` as bare form vs argument form ✅ RESOLVED

**Observation:** The early CSV example uses `@eol` (bare, no parentheses/argument). The spec recommends `@eol(field_name)` where the field is a string field containing the line ending.

**Decision:** All three forms are supported in v1:

- `@eol` — bare form, emits `\n`. Most concise.
- `@eol()` — explicit zero-argument form, emits `\n`. Equivalent to `@eol`.
- `@eol(field)` — emits the field value as a scalar string, followed by `\n`.

As a template language replacement, syntactic economy matters. The parser handles the bare form as a special case (a directive without parentheses), which all three forms normalize to the same `Eol` IR node.

---

## DEC-012 — Render trait design ✅ RESOLVED

**Question:** Should rendering use `fn render(&self) -> String` or `fn render(&self, out: &mut String) -> fmt::Result`?

**Decision:** Use the `out: &mut String` form as the core trait method for zero-allocation recursive rendering. Expose a convenience `fn render(&self) -> String` wrapper. The `Render` trait is:

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
