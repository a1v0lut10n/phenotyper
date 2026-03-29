# Phenotyper v1 — Implementation Plan

## Overview

This document defines the phased implementation plan for the Phenotyper v1 compiler. It is organized into milestones that can be delivered incrementally, with each milestone producing usable, testable output.

The implementation sequence follows the compiler pipeline: **project setup → lexer → parser → AST → semantic analysis → IR → Rust code generation → CLI → end-to-end validation**.

---

## Prerequisites

Before starting implementation:

1. **Resolve open design decisions** — Review [REQ-DECISIONS](../requirements/REQ-DECISIONS-open-design-decisions.md) and lock in choices for DEC-001 through DEC-012.
2. **Update legacy examples** — Align `docs/examples/csv.md` and `csv.pht` to the normative v1 syntax (see DEC-009, DEC-010, DEC-011).
3. **Verify Rustemo availability** — Confirm Rustemo version, API surface, and integration model. Review COLAP implementation for patterns.

---

## Milestone 0 — Project Scaffolding

**Goal:** A buildable Rust workspace with the right crate structure.

### Tasks

- [x] **T-000**: Initialize Rust workspace at project root with `Cargo.toml`.
- [x] **T-001**: Create `crates/phenotyper-core/` crate — the core compiler library (lexer, parser, AST, IR, semantic analysis, codegen).
- [x] **T-002**: Create `crates/phenotyper-cli/` crate — the CLI binary that depends on `phenotyper-core`.
- [x] **T-003**: Set up `tests/` directory structure per [REQ-TEST-006](../requirements/REQ-TEST-testing-strategy.md).
- [x] **T-004**: Add initial `Cargo.toml` dependencies: `rustemo`, `clap` (CLI), `serde` (optional for JSON diagnostics), `thiserror`.
- [x] **T-005**: Create initial test fixture files — valid and invalid `.pht` samples.
- [x] **T-006**: Set up CI configuration (cargo check, cargo test, cargo clippy).

**Requirements covered:** None directly; foundational.

---

## Milestone 1 — Lexer

**Goal:** A working tokenizer that produces the full v1 token set.

### Tasks

- [ ] **T-010**: Define the `Token` enum covering all v1 tokens (REQ-COMP-003).
- [ ] **T-011**: Define `Span` type for source-position tracking (file, start line/col, end line/col).
- [ ] **T-012**: Implement lexer for pure `.pht` input.
- [ ] **T-013**: Implement string literal lexing with escape sequences (`\"`, `\\`, `\n`, `\r`, `\t`).
- [ ] **T-014**: Implement comment handling (line `//` and block `/* */`).
- [ ] **T-015**: Implement Markdown `.md` source extraction — extract `pht` fenced blocks and build source map.
- [ ] **T-016**: Write lexer unit tests: valid tokens, edge cases, malformed input.

**Requirements covered:** REQ-COMP-002, REQ-COMP-003, REQ-LANG-016 through REQ-LANG-020.

---

## Milestone 2 — Parser and AST

**Goal:** Parse v1 syntax into a span-annotated AST using Rustemo.

### Tasks

- [ ] **T-020**: Define Rustemo grammar file from the EBNF in REQ-COMP-004.
- [ ] **T-021**: Define AST data types in Rust (REQ-COMP-005): `FileAst`, `NamespaceAst`, `UsesAst`, `TypeAliasAst`, `EnumDeclAst`, `TypeDefAst`, `FieldDeclAst`, `TypeExprAst`, `RenderExprAst`, `DirectiveAst`.
- [ ] **T-022**: Implement Rustemo actions to produce the AST from parser output.
- [ ] **T-023**: Implement `namespace` and `uses` parsing.
- [ ] **T-024**: Implement `type Name: TypeExpr;` (type alias) parsing.
- [ ] **T-025**: Implement `type Name: [member1, member2];` (enum) parsing.
- [ ] **T-026**: Implement phenotype declaration parsing (singular and `plural` forms).
- [ ] **T-027**: Implement field declaration parsing (`required`/`optional`, type expression with unions and cardinality `+`).
- [ ] **T-028**: Implement render expression parsing: string literals, `@(field)`, `@name(args)`.
- [ ] **T-029**: Implement block directive parsing: `@ifset(field) { ... }`, `@ifnotempty(field) { ... }` — including nested render bodies.
- [ ] **T-030**: Write parser golden tests — valid inputs with expected AST snapshots.
- [ ] **T-031**: Write parser negative tests — malformed syntax with expected error messages.

**Requirements covered:** REQ-COMP-004, REQ-COMP-005, REQ-LANG-001 through REQ-LANG-020, REQ-TEST-001.

---

## Milestone 3 — Symbol Table and Name Resolution

**Goal:** Collect all declared symbols and resolve all references.

### Tasks

- [ ] **T-040**: Define symbol table data structure (REQ-COMP-008).
- [ ] **T-041**: Implement symbol collection pass — walk AST, register all type names (singular, plural, enum, alias).
- [ ] **T-042**: Implement duplicate name detection: singular collisions, plural collisions, cross-collisions.
- [ ] **T-043**: Implement field-name uniqueness checking per type.
- [ ] **T-044**: Implement type-reference resolution — resolve named types to `TypeId`, `EnumId`, or `PrimitiveType`.
- [ ] **T-045**: Implement field-reference resolution in directives — resolve field names within the current type.
- [ ] **T-046**: Implement `uses` import resolution (bring external symbols into scope).
- [ ] **T-047**: Implement shadowing detection (local shadows import → warning).
- [ ] **T-048**: Write symbol/name resolution tests (REQ-TEST-002 scenarios).

**Requirements covered:** REQ-COMP-008, REQ-COMP-009 (symbol validation), REQ-COMP-012.

---

## Milestone 4 — Intermediate Representation and Normalization

**Goal:** Lower the AST into a clean, desugared IR ready for code generation.

### Tasks

- [ ] **T-050**: Define IR data types in Rust (REQ-COMP-006): `PhenotypeModule`, `PhenotypeType`, `FieldDef`, `ValueType`, `RenderNode`, `EnumType`.
- [ ] **T-051**: Implement AST-to-IR lowering pass.
- [ ] **T-052**: Implement directive canonicalization — `@(field)` → `Emit(field_id)`.
- [ ] **T-053**: Implement `@eol` lowering (to `Eol(field_id)` or `Emit(field_id)` per DEC-002).
- [ ] **T-054**: Implement union flattening.
- [ ] **T-055**: Implement primitive type recognition (resolve identifiers like `string`, `int64` to `PrimitiveType`).
- [ ] **T-056**: Implement singular/plural normalization — store plural companion on `PhenotypeType`, lower plural field references to `UserPlural { collection_of }`.
- [ ] **T-057**: Implement cardinality normalization — `+` → `Cardinality::OneOrMore`.
- [ ] **T-058**: Implement `@ifset` lowering — `@ifset(field) { body }` → `IfSet { field: FieldId, body: Vec<RenderNode> }`.
- [ ] **T-059**: Implement `@ifnotempty` lowering — `@ifnotempty(field) { body }` → `IfNotEmpty { field: FieldId, body: Vec<RenderNode> }`.
- [ ] **T-060-a**: Write IR normalization tests (including block directive lowering).

**Requirements covered:** REQ-COMP-006, REQ-COMP-007.

---

## Milestone 5 — Semantic Validation

**Goal:** Validate the IR for full semantic correctness before code generation.

### Tasks

- [ ] **T-060**: Implement type validation — plural companion integrity, union member validity.
- [ ] **T-061**: Implement render validation — directive arity, argument kinds, field resolvability, non-empty render body.
- [ ] **T-062**: Implement cardinality/render compatibility — reject direct `@emit` of plural fields and `+` cardinality fields.
- [ ] **T-063**: Implement optional-emit validation — direct `@emit` of optional fields outside `@ifset` is an error.
- [ ] **T-063a**: Implement `@ifset` type validation — `@ifset` on a non-optional field is an error.
- [ ] **T-063b**: Implement `@ifnotempty` type validation — `@ifnotempty` on a non-collection field is an error.
- [ ] **T-064**: Implement separator type validation — `@join` separator must be singular scalar string.
- [ ] **T-065**: Implement cyclic type graph detection — reject recursive types (REQ-COMP-011).
- [ ] **T-066**: Implement generation validation — ensure all types are mappable to Rust, unions → enum shape.
- [ ] **T-067**: Write semantic validation tests (REQ-TEST-002 scenarios).

**Requirements covered:** REQ-COMP-009 (type, render, generation validation), REQ-COMP-011.

---

## Milestone 6 — Diagnostics

**Goal:** Produce rich, actionable diagnostics for all error and warning conditions.

### Tasks

- [ ] **T-070**: Define `Diagnostic` struct: severity, summary, span, explanation, optional fix suggestion.
- [ ] **T-071**: Define `DiagnosticSeverity` enum: Error, Warning, Info.
- [ ] **T-072**: Integrate diagnostics throughout parser, symbol, and semantic phases (ensure source spans propagate).
- [ ] **T-073**: Implement human-readable diagnostic formatter (stderr).
- [ ] **T-074**: Implement JSON diagnostic formatter (for `--json` flag).
- [ ] **T-075**: Write diagnostic tests — verify correct messages, source locations, and suggestions (REQ-TEST-005).

**Requirements covered:** REQ-COMP-010, REQ-CLI-007.

---

## Milestone 7 — Rust Code Generation

**Goal:** Generate idiomatic Rust code from the validated IR.

### Tasks

- [ ] **T-080**: Implement Rust type mapping: `PrimitiveType` → Rust type, `ValueType` → Rust type reference.
- [ ] **T-081**: Implement singular struct generation (REQ-CODEGEN-002).
- [ ] **T-082**: Implement plural wrapper struct generation with `from_vec`, `into_vec`, `as_slice`, `iter`, `From`/`Into` (REQ-CODEGEN-003).
- [ ] **T-083**: Implement `Render` trait and trait implementations for singular types (REQ-CODEGEN-006).
- [ ] **T-084**: Implement `render` on plural wrappers — join logic per phenotype definition.
- [ ] **T-085**: Implement builder struct generation for singular types (REQ-CODEGEN-004).
- [ ] **T-086**: Implement builder struct generation for plural types (`push`, `extend`).
- [ ] **T-087**: Implement `BuildError` enum generation (REQ-CODEGEN-005).
- [ ] **T-088**: Implement `build()` runtime validation: missing required fields, empty `OneOrMore` collections.
- [ ] **T-089**: Implement union type → Rust enum generation (REQ-CODEGEN-007).
- [ ] **T-090**: Implement namespace-level enum → Rust enum generation (REQ-CODEGEN-008).
- [ ] **T-091**: Implement naming strategy: DSL names → idiomatic Rust names (REQ-CODEGEN-009).
- [ ] **T-092**: Implement module structure: namespace path → Rust module hierarchy (REQ-CODEGEN-012).
- [ ] **T-093**: Implement `@ifset` codegen — generate `if let Some(val) = self.field { ... }` conditional rendering blocks.
- [ ] **T-093a**: Implement `@ifnotempty` codegen — generate `if !self.field.is_empty() { ... }` conditional rendering blocks.
- [ ] **T-094**: Implement `rustfmt` post-processing for generated code.
- [ ] **T-094**: Write codegen snapshot tests — compare generated Rust against golden files (REQ-TEST-003).
- [ ] **T-095**: Write codegen compile tests — ensure generated code compiles.
- [ ] **T-096**: Write codegen runtime tests — ensure rendered output matches expectations.

**Requirements covered:** REQ-CODEGEN-001 through REQ-CODEGEN-012, REQ-TEST-003.

---

## Milestone 8 — CLI

**Goal:** A working `phenotyper` binary with `check` and `build` subcommands.

### Tasks

- [ ] **T-100**: Set up `clap` command structure with `check` and `build` subcommands (REQ-CLI-002).
- [ ] **T-101**: Implement `check` — run full pipeline except code generation, emit diagnostics.
- [ ] **T-102**: Implement `build --out <path>` — run full pipeline including code generation, write output.
- [ ] **T-103**: Implement `dump-ast` subcommand (REQ-CLI-003).
- [ ] **T-104**: Implement `dump-ir` subcommand (REQ-CLI-003).
- [ ] **T-105**: Implement `--json` flag for machine-readable diagnostics (REQ-CLI-004).
- [ ] **T-106**: Implement exit code policy: 0 success, 1 failure, 2 usage error (REQ-CLI-005).
- [ ] **T-107**: Implement file type detection by extension (REQ-CLI-006).
- [ ] **T-108**: Write CLI integration tests.

**Requirements covered:** REQ-CLI-001 through REQ-CLI-007.

---

## Milestone 9 — End-to-End Validation

**Goal:** Prove the system with complete examples from source to rendered output.

### Tasks

- [ ] **T-110**: Update `docs/examples/csv.pht` and `csv.md` to normative v1 syntax.
- [ ] **T-111**: Create structured prompt example (second end-to-end test case).
- [ ] **T-112**: Create union-heavy example (third end-to-end test case).
- [ ] **T-113**: Create end-to-end test: CSV `.pht` → compile → generate Rust → compile Rust → render → verify output.
- [ ] **T-114**: Create end-to-end test: same flow from `.md` container.
- [ ] **T-115**: Create end-to-end test: structured prompt flow.
- [ ] **T-116**: Create end-to-end test: union-heavy flow.
- [ ] **T-117**: Create optional-field example using `@ifset` and `@ifnotempty` (e.g., CSV with optional header/footer).
- [ ] **T-118**: Create end-to-end test: optional-field flow.
- [ ] **T-119**: Document worked examples in `docs/examples/`.

**Requirements covered:** REQ-TEST-004.

---

## Milestone 10 — Documentation and Polish

**Goal:** Complete documentation and prepare for release.

### Tasks

- [ ] **T-120**: Write `docs/howto/authoring_phenotypes.md` — guide for writing phenotype definitions.
- [ ] **T-121**: Write `docs/howto/using_generated_code.md` — guide for using generated Rust APIs.
- [ ] **T-122**: Write `docs/howto/compiler_usage.md` — guide for using the CLI.
- [ ] **T-123**: Finalize README with accurate installation and usage instructions.
- [ ] **T-124**: Run `cargo clippy` and address all warnings.
- [ ] **T-125**: Performance sanity check: profile compile and render for the CSV example.
- [ ] **T-126**: License selection and SPDX header.

**Requirements covered:** Cross-cutting.

---

## Dependency Graph

```
M0 (Scaffolding)
 └─→ M1 (Lexer)
      └─→ M2 (Parser/AST)
           └─→ M3 (Symbol Table/Name Resolution)
                └─→ M4 (IR/Normalization)
                     └─→ M5 (Semantic Validation)
                          ├─→ M6 (Diagnostics)    ← can start with M3
                          └─→ M7 (Code Generation)
                               └─→ M8 (CLI)
                                    └─→ M9 (E2E Validation)
                                         └─→ M10 (Docs/Polish)
```

Note: M6 (Diagnostics) can be worked on incrementally alongside M3–M5, since each phase produces diagnostics. However, it becomes testable as a complete unit after M5.

---

## Suggested Implementation Order (Critical Path)

| Priority | Milestone | Estimated Effort | Rationale |
|----------|-----------|-----------------|-----------|
| 1 | M0 Scaffolding | 1 day | Unblocks everything |
| 2 | M1 Lexer | 2–3 days | Foundation for parser |
| 3 | M2 Parser/AST | 5–7 days | Core complexity, Rustemo integration |
| 4 | M3 Symbols/Resolution | 3–4 days | Unlocks semantic analysis |
| 5 | M4 IR/Normalization | 3–4 days | Clean representation for codegen |
| 6 | M5 Semantic Validation | 3–4 days | Catch errors before generation |
| 7 | M6 Diagnostics | 2–3 days | (Incremental alongside M3–M5) |
| 8 | M7 Code Generation | 7–10 days | Largest single milestone |
| 9 | M8 CLI | 2–3 days | Wraps the pipeline |
| 10 | M9 E2E Validation | 3–5 days | Proves the system |
| 11 | M10 Docs/Polish | 3–5 days | Release readiness |

**Total estimated:** ~35–50 working days for a solo developer.
