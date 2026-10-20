---
date: 2026-10-20
type: task
status: done
tags: [language, annotations, PHT-0028]
---

# PHT-0028 slice 1 — the grammar and the checks

Design: `docs/design/2026-10-20-elasticity-volatility-and-segments.md`.
(Recorded with slice 2 — the doc fell out of slice 1's commit chain, a
shell `&&`-after-`grep -c` trap, caught during P2.)

## Actions

- Grammar: field annotation clauses — `volatile`, `segment("name")`,
  `elastic(drop, priority N)`, `elastic(truncate, marker "…")` — with
  keyword terminals and `IntLiteral`; rustemo regenerated, the field
  action threaded into `FieldDecl`.
- IR: `FieldDef.annotations: FieldAnnotations`, duplicates refused at
  lowering.
- Checks: `elastic(drop)` demands an unset-shaped field (optional —
  the existing optional-outside-@ifset check then guards every
  reference — or a collection); `elastic(truncate)` demands a scalar
  string; STABLE-AFTER-VOLATILE is a compile error, transitive through
  same-module types, literal text exempt (the ClockLine shape).

## Verified

Inertness golden (then at whole-code strength; P2 moved it to the
designed render-path strength), the semantic pins, the uses-carry
golden, 255 tests green.
