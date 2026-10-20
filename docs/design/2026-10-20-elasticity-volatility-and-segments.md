# PHT-0028 — elasticity, volatility and segments: one annotation pass

*The design pass item 3 of
`docs/incubation/2026-09-22-context-assembly-enhancements.md` asked
for, graduated with its 2026-10-19 synthesis. Three annotation
dimensions want the same nodes; this record designs them together so
no dimension is retrofitted.*

The week that ripened it: genite's GEN-080 settled that the lever in
agent context work is what the engine is shown, not what the disk
costs; GEN-081 landed the prefix store and proved — by a one-digit
byte-diff — that a single volatile line mid-prompt forfeits every
stable token after it; and the capacity competition now has a concrete
use for a prompt that knows its own floor. Phenotyper is where all
three contracts belong: the template is the one place that knows
which parts bend, which parts change, and where a cacheable unit ends.

## The binding guarantee, first

**Strict determinism survives, both paths.** Annotations are INERT
under plain `render()`: a phenotype carrying every annotation in this
record renders byte-identically to its unannotated twin — code
generation's contract, pinned by a golden test that compiles the same
template with and without annotations and compares bytes. The budgeted
renderer is itself a pure function of (field values, budget, measure):
no clock, no environment, no iteration-order hazard may enter either
path. Elasticity adds a second deterministic renderer, never a
nondeterministic one.

## D1 — the annotations (field clauses, compile-checked)

Annotations attach to FIELD DECLARATIONS, after the type, in the
grammar's existing clause position:

    SearchEvidence:
        hits: required Hits elastic(drop, priority 2),
        guidance: optional string elastic(drop, priority 1),
        page: required string elastic(truncate, floor 2000, marker " […]"),
        clock: required string volatile,
        skills: required SkillEntries segment("skills"),
        ...template runs unchanged...
    ;

- **`elastic(drop, priority N)`** — the field's contribution may
  vanish whole. Compile check: every template reference to a `drop`
  field sits inside `@ifset(field)` (or the field is a plural, whose
  join degrades by dropping TAIL items first — the caller orders the
  list, so "the tail is cheapest" is the caller's contract). Dropping
  renders exactly as unset does today — no new text machinery.
- **`elastic(truncate, marker "…")`** — a string field may be cut,
  marker appended. The FLOOR LIVES AT RENDER TIME, in the caller's
  measure units (decision 2): the template authors what may bend and
  how the cut reads; the caller decides how far it may bend per call.
  The cut point for a non-char measure is found by binary search on
  the measure — deterministic for any deterministic measure.
- **`volatile`** — the field's content changes between renders that
  mean the same thing (clocks, live numbers). Compile check: within a
  phenotype, no stable reference may follow the first volatile one
  (stable-before-volatile BY CONSTRUCTION — the GEN-081 lesson, made a
  compile error instead of a code review find). Volatility propagates
  from nested phenotypes.
- **`segment("name")`** — the field's rendered span is a cacheable
  unit. No hashing in phenotyper: the generated API exposes named byte
  ranges; the consumer brings its own hash, as genite's stores already
  do.

Priorities are small integers; LOWER drops first. Ties break by
template order, later first — the tail is cheapest, everywhere.

## D2 — the generated API

`render()` is untouched. Annotated phenotypes additionally generate:

    fn render_budgeted(&self, budget: &Budget<'_>) -> Budgeted;

    struct Budget<'a> {
        limit: usize,
        measure: &'a dyn Fn(&str) -> usize,
        /// Truncation floors in MEASURE units (decision 2): a uniform
        /// default plus per-field overrides; absent means floor 0
        /// (a truncatable field may vanish to its marker).
        floors: Floors<'a>,
    }

    struct Budgeted {
        text: String,
        cost: usize,          // measure(text), the final truth
        full_cost: usize,     // measure of the undegraded render
        floor_cost: usize,    // measure with everything elastic spent
        dropped: Vec<&'static str>,      // field names, in drop order
        truncated: Vec<&'static str>,
        stable_len: usize,    // bytes before the first volatile content
        segments: Vec<(&'static str, core::ops::Range<usize>)>,
    }

The MEASURE IS THE CALLER'S (the synthesis's seam): genite passes its
tokenizer's counter, tests pass `str::len`, and the crate keeps no
tokenizer. Because token measures are not additive across boundaries,
the algorithm re-measures the WHOLE candidate text after each
degradation step — the budget guarantee (`cost <= budget`, or the
floor reached and reported) holds for ANY measure, at the price of at
most one measure call per elastic field. `full_cost` and `floor_cost`
are what the capacity competition's degrade-to-fit admission wants:
a TurnShape can carry both and let selection price "run degraded now"
against "run whole later".

## D3 — the degradation algorithm (deterministic, node-aligned)

1. Render in full; if `measure(text) <= budget`, done — zero
   degradation, byte-identical to `render()`.
2. Otherwise, in priority order (lowest first, ties later-first):
   drop `drop` fields whole; for plurals, drop tail items one at a
   time. Re-measure after each step; stop when within budget.
3. Still over: truncate `truncate` fields tail-first to their floors,
   marker appended. Re-measure each step.
4. Still over: return the floor render with `cost > budget` reported
   honestly — the caller decides (route elsewhere, or send anyway).

Every cut lands on a node boundary (a whole field, a whole plural
item, or a declared char floor) — never mid-text re-flow — so the
prompt-cache splice rung, if it ever earns a ticket, inherits
segment-aligned degradation for free.

## Non-goals, re-affirmed

No tokenizer dependency. No runtime template engine (build-time
codegen stays). No reordering of authored content — `volatile` VALIDATES
order, it never rewrites it; the author owns the template. No LLM or
provider awareness.

## Slices

- **P1 — the grammar and the checks**: annotation clauses parse into
  the AST; plain `render()` output byte-identical (the golden);
  compile errors for unguarded `drop` references and stable-after-
  volatile; `uses`-imported phenotypes carry their annotations.
- **P2 — the budgeted renderer**: `render_budgeted`, `Budgeted`,
  the algorithm above, `stable_len`, `segments`; property-style tests
  (any budget, any measure: determinism, node alignment, the floor
  honesty).
- **P3 — the first consumer** (genite, its own GEN ticket):
  `ContextBudget` becomes `render_budgeted` calls; the TurnShape
  carries full and floor needs; the clock field annotated `volatile`
  so the GEN-081 arrangement is compiler-enforced instead of
  comment-enforced.

## Decisions (owner, 2026-10-20)

1. **Bare clauses** (`elastic(...)`, `volatile`, `segment(...)`) — no
   `@` prefix; the `@` family stays template-side.
2. **Floors live at render time, in measure units** — the template
   authors what may bend and how the cut reads (the marker); the
   caller decides how far, per call, through `Budget::floors`. More
   flexible, and it keeps the template free of measurement
   assumptions entirely.
3. **The v1 grammar is the base.** Namespace RESOLUTION already landed
   (PHT-0027, `uses` with cross-namespace codegen, 0.3.0); the v2
   STRUCTURAL syntax remains a draft. The clauses fit both, so they
   ride into v2 unchanged whenever it lands — nothing waits.
