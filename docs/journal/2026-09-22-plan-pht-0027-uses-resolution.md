---
date: 2026-09-22
type: journal
components: []
aspects: []
tasks:
  - docs/tasks/2026-09/2026-09-22-uses-resolution.md
---

# Plan: PHT-0027 — resolve `uses`, cross-namespace imports end to end

- **When:** 2026-09-22

## Context

The first consumer-driven ticket since the v2 freeze (2026-04). Genite
intends to re-engage phenotyper for typed, budget-aware context
assembly in its agent (genite
`docs/incubation/2026-09-22-phenotyper-typed-context-assembly.md`; it
used phenotyper this way once, in its pre-GEN-013 brain), and the
first thing that engagement needs is a shared prompt vocabulary — a
common `aivolution/ai/prompt` namespace imported by genite, aicogito's
"prompts as phenotypes" and whoever comes next. `uses a/b/c;` has been
dead syntax past the parser since v2 shipped: accepted by the grammar
and lexer, carried by the AST, consumed by nothing, while
REQ-LANG-003 has specified its semantics all along. Graduated from
`docs/incubation/2026-09-22-context-assembly-enhancements.md` (item 1
of 3; the parse round-trip and budget annotations stay incubating).

## Decisions

- **Compilation grows to a set.** `uses` needs a second file, so the
  compiler gains search roots and a namespace ↔ file mapping that
  mirrors the codegen mapping in reverse; dependencies are resolved,
  parsed and compiled as needed, with cycles a hard error. The
  single-file `compile()` / `compile_source()` API stays untouched —
  a file without `uses` compiles exactly as today, and a file with
  `uses` but no roots gets a clear diagnostic, not a behaviour change.
- **REQ-LANG-003 enforced as written**: imported declarations enter
  unqualified scope; a local declaration shadowing an import is a
  warning carrying both locations; the same unqualified name from two
  imports is a hard error suggesting qualification, while fully
  qualified references always work.
- **Whole-namespace imports only.** Selective imports
  (`uses a/b/c::{X}`) and re-exports are a later requirement;
  cross-crate/registry resolution is out of scope — roots are local
  directories.
- **Ticket housekeeping**: `docs/NEXT-TICKET` bootstrapped at
  PHT-0028 (this repo predated the numbering convention; PHT-1…26
  stay unpadded, mirroring genite's historical note).

## Next

Implementation on `feature/PHT-0027-uses-resolution`: the compilation
set, symbol resolution, cross-namespace codegen (Rust `use` paths into
the already namespace-shaped out tree), located diagnostics in human
and JSON, e2e two-namespace fixtures in the rustc-compiled style, and
the README/example updates the task's `affects` demands.
