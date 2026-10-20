---
date: 2026-10-20
type: journal
components: []
aspects: []
tasks:
  - docs/tasks/2026-10/2026-10-20-pht-0028-1-the-grammar-and-checks.md
  - docs/tasks/2026-10/2026-10-20-pht-0028-2-the-budgeted-renderer.md
---

# Execution: PHT-0028 — the annotation language and the budgeted renderer; v0.4.0

- **When:** 2026-10-20 15:20 local, both crates live on crates.io.

## Context

The design pass item 3 of the context-assembly enhancements asked for,
ripened by a week of genite field work: GEN-081's prefix store proved —
by a one-digit byte-diff — that a single volatile line mid-prompt
forfeits every stable token after it, and the capacity competition
gained a concrete use for a context that knows its own floor. The
owner's three decisions shaped the syntax (bare clauses; truncation
floors at render time in the caller's measure units; the v1 grammar as
base, namespace resolution already being live), and one requirement was
made binding before any code: STRICT DETERMINISM SURVIVES — annotations
inert under plain `render()`, the budgeted path itself a pure function.
Design record: `docs/design/2026-10-20-elasticity-volatility-and-segments.md`.

## Details

**Slice 1 — the grammar and the checks** (#27). Field declarations take
`volatile`, `segment("name")`, `elastic(drop, priority N)` and
`elastic(truncate, marker "…")`; rustemo regenerated the parser and
appended the action plumbing; the IR carries `FieldAnnotations` with
duplicates refused at lowering. The contracts became compile errors:
a `drop` field must render as unset does (optional — the existing
optional-outside-`@ifset` rule then guards every reference — or a
collection), `truncate` demands a scalar string, and
STABLE-AFTER-VOLATILE REFUSES TO BUILD — transitive through
same-module types, literal text exempt so a volatile unit may finish
its own sentence (the ClockLine shape). The inertness golden pinned at
its strongest form: byte-identical generated code, within a module and
across a `uses` set. Two language facts were paid for while testing:
every `.pht` opens with a namespace header and closes with a lone `.`.

**Slice 2 — the budgeted renderer** (#28). Annotated types generate
`render_budgeted(&self, &Budget) -> Budgeted` beside an untouched
`render()`: `Budget` carries the limit, THE CALLER'S measure, and
floors in measure units (uniform default, per-field overrides);
`Budgeted` answers with text, cost, full_cost, floor_cost, dropped,
truncated, `stable_len` (the KV-prefix boundary) and `segments`
(named byte ranges). The driver re-measures the whole candidate after
every degradation step, so the budget guarantee holds for ANY
deterministic measure — pinned under a word-count measure; drops run
by priority ascending (later-in-template first), plural tails and
truncation cuts by binary search with the render-time floor clamped
through the field's own measured span; an impossible budget returns
the floor render with `cost > limit` reported — honesty, never
silence. The P1 golden moved to its designed strength (the `impl
Render` blocks byte-identical, both twins rendering the same bytes),
and the contracts golden EXECUTES generated code through the full
gauntlet. Slice 1's task record, which had fallen out of its commit
chain (a shell `grep -c` short-circuit), was recovered here.

**Documentation and the release.** README gained the budget-aware
CONTEXT generation section — the owner's word, truer than "prompt":
the machinery operates on the assembled context — with REQ-LANG-021
and REQ-CODEGEN-013 pinning the clauses and the generated surface.
v0.3.1 was tagged first as the pre-annotations mark (and the first
tagged state carrying the instantiated 2008-2026 copyright); v0.4.0
tagged on the release merge; `phenotyper` and `phenotyper-cli` 0.4.0
published to crates.io — the copyright's first appearance in the
published artifacts.

**Named for the next chapter:** P3, the genite consumer — bump the
`phenotyper = "0.3"` pin, annotate the context assembler's fields
(the clock becomes `volatile`, making GEN-081's arrangement
compiler-enforced), and let the TurnShape carry full and floor needs
into the capacity competition's degrade-to-fit admission.
Cross-namespace volatility metadata rides along when `uses` carries
annotations further than inertness.

## Links

- docs/design/2026-10-20-elasticity-volatility-and-segments.md
- PRs #26 (design), #27 (P1), #28 (P2 + docs), #29 (v0.4.0); tags
  v0.3.1, v0.4.0
- genite docs/implementation/2026-10-20-gen-081-the-prefix-store.md —
  the field lesson the `volatile` check encodes
- aicogito docs/incubation/2026-10-18-reflection-reads-and-lancedb.md
  — the measurement arc that reframed what context work costs
