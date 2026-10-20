---
date: 2026-10-20
type: task
status: done
tags: [codegen, budget, PHT-0028]
---

# PHT-0028 slice 2 — the budgeted renderer

Design: `docs/design/2026-10-20-elasticity-volatility-and-segments.md`.

## Actions

- `codegen::budget`: annotated types additionally generate
  `render_budgeted(&self, &Budget) -> Budgeted` plus an instrumented
  twin of the render walk reusing the standard emitters for plain
  nodes; parent-context types wait for the composition work. A
  once-per-module runtime carries `Budget` (limit, the CALLER'S
  measure, floors in measure units with per-field overrides) and
  `Budgeted` (text, cost, full_cost, floor_cost, dropped, truncated,
  stable_len, segments).
- The driver re-measures the whole candidate after every step (the
  guarantee holds for any deterministic measure); drops by priority
  ascending, later-in-template first; plural tails and truncation cuts
  by binary search with the render-time floor clamped through the
  field's own measured span; an impossible budget returns the floor
  render with `cost > limit` reported — honesty, never silence.
- `stable_len` marks the byte before the first volatile content
  (transitive); `segment` fields report exact byte ranges.
- The inertness goldens moved to their designed strength: `impl
  Render` blocks byte-identical, both twins rendering the same bytes,
  within a module and across a `uses` set.

## Verified

The contracts golden EXECUTES generated code: roomy-budget identity,
determinism, stable_len at the clock, the title segment's span,
priority-1 drop alone, tail drop keeping early items, marker
truncation within budget, floor honesty with a per-field floor held,
and the budget guarantee under a non-additive word measure. Full
gates: fmt, clippy `-D warnings` all targets, 256 tests green.
