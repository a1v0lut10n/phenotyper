---
date: 2026-09-22
type: task
status: done
affects:
  - README.md
tags: [language, uses, imports, resolution, codegen, PHT-0027]
---

# PHT-0027: resolve `uses` — cross-namespace imports, end to end

*Status 2026-09-22, as built: all six deliverables, with three deviations
worth recording. (1) REQ-LANG-003's "hard errors **unless fully
qualified**" is unenforceable as written — the grammar has no qualified
syntax in type positions (type names are bare identifiers), so
cross-import ambiguity is a plain hard error suggesting a rename;
qualified type references need a grammar regeneration and stay future
work. (2) Imported types inside union types are rejected with an honest
diagnostic — union variant naming is local-only for now. (3) The design
memo needed no edit: its "import" mentions are the template-import CLI,
not `uses`. API landed as `compile_with_roots(entry, roots, out_dir) ->
Vec<CompileOutput>` plus `--root` on `check`/`build`; the set writer
emits the intermediate `mod.rs` chain so the tree mounts with one `pub
mod`, and imported rendering goes through the exporter's `Render` trait
via UFCS. Proven end to end: the two-namespace fixture compiles under
`rustc` and renders `Genite [gold]` across the boundary.*

## Objective

Implement REQ-LANG-003. `uses a/b/c;` is accepted by the grammar and
lexer, carried by the AST (`NamespaceScope.uses`), and consumed by
nothing — dead syntax past the parser. After this ticket a phenotype
in one namespace can use types and phenotypes declared in another,
with the requirement's three rules enforced: imported declarations
enter unqualified scope, local declarations shadow imported ones with
a **warning**, and ambiguous imported names are **hard errors** unless
fully qualified.

## Context

Graduated from
`docs/incubation/2026-09-22-context-assembly-enhancements.md` (item 1
of 3; the parse round-trip and budget annotations stay incubating).
The consumer pressure is genite's planned re-engagement for typed
context assembly (genite
`docs/incubation/2026-09-22-phenotyper-typed-context-assembly.md`):
without `uses` there is no shared `aivolution/ai/prompt` vocabulary —
every consumer copies definitions instead of importing them, and
aicogito's "prompts as phenotypes" work cannot share a namespace with
genite's.

Today the compiler is strictly one-namespace-per-file:
`compile(source_path, out_dir)` reads a single `.pht`/`.md`, and
nothing in `symbol/` (collect.rs, resolve.rs), `ir/lower.rs` or
`semantic/validate.rs` looks at the parsed `uses` declarations.

## Deliverables

### 1. A compilation-set model

`uses` needs a second file to resolve against, so compilation grows
from one file to a set:

- A namespace → source-file mapping that mirrors the existing codegen
  mapping in reverse (`aivolution/format/csv` ↔
  `<root>/aivolution/format/csv.pht` or the `.md` container), rooted
  at one or more search roots.
- API: `compile()` and `compile_source()` keep working unchanged for
  files without `uses`; a file with `uses` compiled without roots gets
  a clear diagnostic. New surface (name to taste at implementation):
  `compile_with_roots(source, roots, out_dir)` or a `CompileSet`
  builder — resolving, parsing and compiling dependencies as needed.
- Cycle detection: `a uses b uses a` is a hard error naming the cycle.

### 2. Symbol resolution per REQ-LANG-003

- Imported declarations (types and phenotypes) enter the importing
  file's unqualified scope in `symbol::resolve`.
- Local shadows imported → warning diagnostic with both locations.
- The same name from two imports → hard error on unqualified use,
  suggesting qualification; fully-qualified references
  (`a/b/c/Name`) always work.

### 3. Codegen across namespaces

Generated Rust for the importing namespace references the imported
namespace's generated module (the out_dir tree is already
namespace-shaped): emit Rust `use` paths rather than duplicating
definitions, and ensure one `compile` of a set writes each namespace's
module once.

### 4. Diagnostics

Unknown namespace in `uses`, unresolvable file for a namespace, the
shadow warning, the ambiguity error and the cycle error — all with
`SourceLocation`, in both human and JSON formats.

### 5. Tests and fixtures

- Unit tests per phase (symbol, ir, semantic) for the three REQ rules.
- E2E in the existing style (generated code compiled by `rustc` and
  rendered): a two-namespace fixture where a prompt phenotype uses a
  shared type from `aivolution/core/…`, plus shadow, ambiguity and
  cycle fixtures asserting exact diagnostics.
- CLI: `check` and `build` on a multi-file set (search-root flag).

### 6. Docs

README language section documents working imports with an example;
the design memo's `uses` mentions updated from "future" to shipped;
`docs/examples/` gains a two-namespace example.

## Out of scope

- The parse half of the round-trip and budget annotations (still
  incubating, same entry).
- Re-exports / selective imports (`uses a/b/c::{X}`) — REQ-LANG-003
  imports whole namespaces; refinements are a later requirement.
- Registry/dependency resolution across crates — roots are local
  directories; cross-crate sharing ships source files.

## Completion

Before `status: done`: all three REQ-LANG-003 rules enforced with
tests, the e2e two-namespace fixture green, README in `affects`
describing working imports, and a linking journal entry.

---

*Housekeeping: this ticket bootstraps `docs/NEXT-TICKET` (PHT-0028
next). Historical PHT-1…PHT-26 predate zero-padding and stay as they
are, mirroring genite's convention note.*
