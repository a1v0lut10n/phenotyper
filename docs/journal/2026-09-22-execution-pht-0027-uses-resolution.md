---
date: 2026-09-22
type: journal
components: []
aspects: []
tasks:
  - docs/tasks/2026-09/2026-09-22-uses-resolution.md
---

# Execution: PHT-0027 — `uses` resolved, end to end; v0.3.0

- **When:** 2026-09-22

## Context

Implementation of the same-day plan: `uses a/b/c;` had been dead syntax
past the parser since v2 — accepted by grammar, lexer and AST, consumed
by nothing — while REQ-LANG-003 specified its semantics. The consumer
pressure is genite's typed-context re-engagement, which needs a shared
prompt vocabulary across namespaces.

## Details

- **The export/import handshake.** A compiled namespace hands its
  importers a `NamespaceExports` (name → kind: phenotype, plural, enum,
  alias) — everything resolution and codegen need, nothing more,
  because imported declarations are referenced by Rust path, never
  re-emitted. `symbol::build_with_imports` populates the import scope
  *before* collection, so local declarations can warn when they shadow
  an import (both REQ rules land here: shadow → warning naming the
  exporter; a name from two `uses` → hard error at the use site).
- **IR and codegen.** `ValueType::Imported { namespace, name, kind }`
  flows through lowering and validation (imported plurals count as
  collections; imported types inside unions are rejected honestly —
  union variant naming stays local for now). Codegen references imports
  by relative path: `super::` up to the common ancestor, segments down
  (`super::super::core::badge::Badge`), which holds wherever the tree
  mounts. Rendering crosses the boundary via UFCS through the
  *exporter's* `Render` trait — every generated module defines its own
  `Render`, so a plain method call would not resolve in the importer;
  `{prefix}Render::render_into(&self.field, out)` needs no trait
  import.
- **The compilation set.** `compile_with_roots(entry, roots, out_dir)`
  resolves each used namespace at `<root>/<ns>.pht|.md`, compiles every
  namespace once in dependency order (entry last), detects cycles by
  name, verifies a located file declares the namespace it was found
  for, and writes the intermediate `mod.rs` chain so the whole tree
  mounts with a single `pub mod`. `compile()`/`compile_source()` are
  unchanged for files without `uses` and honest about files with them
  (each unresolved `uses` is an error pointing at roots). The CLI
  gains `--root` (repeatable) on `check` and `build`.
- **Deviations from the plan, recorded in the task.** (1) REQ-LANG-003's
  "hard error **unless fully qualified**" is unenforceable as written:
  the grammar has no qualified syntax in type positions, so ambiguity
  is a plain hard error suggesting a rename — qualified references need
  a grammar regeneration and stay future work. (2) Imported types in
  unions are rejected with a diagnostic rather than mis-generated.
  (3) The design memo needed no edit — its "import" mentions are the
  template-import CLI, not `uses`.

## Verification

- 15 new tests: six symbol-scope unit tests (resolution, shadowing,
  ambiguity, unresolved/duplicate/self `uses`), six compilation-set
  integration tests, three CLI tests over a committed two-namespace
  fixture. The flagship proof compiles the generated tree with `rustc`
  and renders `Genite [gold]` across the namespace boundary.
- Full suite 271 passing; `cargo fmt --check` and
  `clippy --workspace --all-targets -D warnings` clean.

## Release: v0.3.0

`ValueType` gained a public variant — technically breaking for
exhaustive matches — so the feature ships as a 0.x minor bump, both
crates (`phenotyper`, `phenotyper-cli`) to crates.io after merge.
`cargo publish --dry-run` packages cleanly. Riding along: the README's
first example dropped a `Visibility` enum nothing in it used (the
second example keeps its own as the enum-syntax illustration).

## Next

Tag `v0.3.0` and publish after merge. Then genite's shared-vocabulary
adoption can begin; the remaining incubation items (the parse half of
the round-trip, budget annotations) stay parked in
`docs/incubation/2026-09-22-context-assembly-enhancements.md`.
