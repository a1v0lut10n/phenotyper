# REQ-INDEX — Requirements Index

## Status
v1 — Master index of all v1 requirements documents.

---

## Document Inventory

| Document | Scope | Requirements |
|----------|-------|--------------|
| [REQ-LANG](REQ-LANG-language-core.md) | Core language syntax, type system, render model | REQ-LANG-001 through REQ-LANG-020 |
| [REQ-COMP](REQ-COMP-compiler-pipeline.md) | Compiler phases, AST, IR, normalization, validation | REQ-COMP-001 through REQ-COMP-012 |
| [REQ-CODEGEN](REQ-CODEGEN-rust-code-generation.md) | Rust code generation: structs, builders, renderers | REQ-CODEGEN-001 through REQ-CODEGEN-012 |
| [REQ-CLI](REQ-CLI-command-line-interface.md) | CLI binary, subcommands, flags, exit codes | REQ-CLI-001 through REQ-CLI-007 |
| [REQ-TEST](REQ-TEST-testing-strategy.md) | Testing strategy: parser, semantic, codegen, e2e | REQ-TEST-001 through REQ-TEST-006 |
| [REQ-DECISIONS](REQ-DECISIONS-open-design-decisions.md) | Open design decisions with recommendations | DEC-001 through DEC-012 |

---

## Coverage by Spec Section

| Spec Section | Requirements |
|--------------|-------------|
| §2 Goals/Non-Goals | REQ-LANG-001 through REQ-LANG-020 |
| §5 Surface Language | REQ-LANG-001 through REQ-LANG-020 |
| §6 Lexical Elements | REQ-LANG-016 through REQ-LANG-020, REQ-COMP-003 |
| §7 Type System | REQ-LANG-006 through REQ-LANG-012 |
| §8 Field Declarations | REQ-LANG-008 |
| §9–10 Render Model & Directives | REQ-LANG-013 through REQ-LANG-015 |
| §11 Name Resolution | REQ-COMP-012 |
| §12 Grammar | REQ-COMP-004 |
| §13 AST | REQ-COMP-005 |
| §14 IR | REQ-COMP-006 |
| §15 Normalization | REQ-COMP-007 |
| §16 Validation | REQ-COMP-009 |
| §17 Diagnostics | REQ-COMP-010 |
| §18 Compiler Pipeline | REQ-COMP-001, REQ-COMP-002 |
| §19 Rust Code Gen | REQ-CODEGEN-001 through REQ-CODEGEN-012 |
| §20 Runtime Semantics | REQ-CODEGEN-006, REQ-CODEGEN-011 |
| §20 (late) Enums | REQ-LANG-005, REQ-CODEGEN-008 |
| §21 Standard Library | REQ-LANG-015 |
| §23 CLI | REQ-CLI-001 through REQ-CLI-007 |
| §24 Testing | REQ-TEST-001 through REQ-TEST-006 |
| §26–27 Open Decisions | DEC-001 through DEC-012 |

---

## Design Document Traceability

| Source Document | Location |
|-----------------|----------|
| Design Memo | [phenotyper_design_memo.md](../design/phenotyper_design_memo.md) |
| V1 Language & Compiler Spec | [phenotyper_v1_language_and_compiler_spec.md](../design/phenotyper_v1_language_and_compiler_spec.md) |
| Project README | [README.md](../../README.md) |
| CSV Examples | [csv.md](../examples/csv.md), [csv.pht](../examples/csv.pht) |
| Roadmap Voice Notes | [2026-03-21-roadmap.md](../voice-notes/2026-03-21-roadmap.md) |
