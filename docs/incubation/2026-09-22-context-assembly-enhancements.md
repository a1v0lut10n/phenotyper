# Enhancements for agent context assembly — a named consumer returns

Genite intends to re-engage phenotyper for its agent's context
assembly (genite `docs/incubation/2026-09-22-phenotyper-typed-context-assembly.md`):
system prompts, voice user messages and retrieval evidence blocks
declared as phenotypes, assembled through the generated builders, with
byte-stable rendering feeding the brain's KV prefix reuse. Genite used
phenotyper this way once (the pre-GEN-013 brain compiled
`context.pht` into typed routing-prompt builders) and the pattern
worked; what follows is what a fuller engagement needs that v0.2.0
does not have, roughly in order of leverage.

1. **Resolve `uses`.** *Graduated 2026-09-22 to
   `docs/tasks/2026-09/2026-09-22-uses-resolution.md` (PHT-0027).*
   The grammar and lexer accept `uses a/b/c;`, the
   AST carries it, and nothing consumes it —
   `REQ-LANG-language-core.md` specifies unqualified-scope imports,
   unimplemented. Without it there is no shared prompt vocabulary: a
   common `aivolution/ai/prompt` namespace reused by genite's agent,
   aicogito's "prompts as phenotypes" and others cannot be expressed;
   every consumer copies definitions instead of importing them.

2. **The parse half of the round-trip.** "Eventually parse" has been
   the design memo's promise since the start. An agent wants it more
   than anyone: the same phenotype that renders a structured request
   should parse the model's structured reply into typed values
   (genite's old `<route>…</route>` contract was rendered by
   phenotyper and parsed by hand-sliced strings). Even a subset —
   phenotypes whose formatters are unambiguous — would close the loop
   for output contracts.

3. **Budget annotations.** Context assembly under a token budget needs
   the template to say which parts bend: per-field or per-section
   elasticity (`droppable`, `truncatable`, a priority order) that the
   generated renderer exposes — render-with-budget returning what was
   dropped, rather than phenotyper learning about tokenizers. The
   intelligence (what to select, how to score) stays in the consumer;
   the phenotype carries the typed contract for degradation. This is
   new language surface and deserves a design pass before syntax.

## 2026-10-19 — the synthesis from genite's October

Item 3 ripened while genite built around phenotyper's byte-stability
(the embed arc, GEN-080, and the prompt-cache incubation). Four
findings for the design pass, so it starts from the October
understanding:

1. **The budget is a caller-supplied measure.** No tokenizer in
   phenotyper — `render_with_budget` takes a measure closure
   (`fn(&str) -> usize`); genite passes its tokenizer's counter,
   aicogito its own, tests pass `str::len`. Budgets become token-true
   while the crate stays measurement-agnostic.

2. **Three annotation dimensions, one syntax pass.** Elasticity (what
   may shrink: `droppable`, `truncatable` with a floor and marker, a
   priority order) is one of THREE annotation classes the same nodes
   now want: VOLATILITY (genite's prompt-cache note: a volatile clock
   line mid-prompt invalidates a cross-session KV prefix store —
   stable-before-volatile ordering maximises the prefix BY
   CONSTRUCTION) and SEGMENT IDENTITY (a cacheable unit's boundary =
   a phenotype node; the content hash is the cache key). Designing
   elasticity's syntax alone retrofits the other two later; one pass
   should hold all three.

3. **Degrade-to-fit is an admission-policy primitive, not a
   truncation nicety.** A turn whose context need exceeds a brain's
   admissible context can today only wait or route elsewhere; a
   phenotype that exposes its FLOOR need (everything droppable
   dropped) gives the capacity competition a third option — run
   degraded now against run whole later — priced like everything
   else. The generated renderer reporting (full_need, floor_need,
   Degradations) is the whole interface the competition requires.

4. **Degradation must be deterministic and byte-stable** for a given
   (content, budget, measure) — dropping at node boundaries, never
   mid-text re-flow — or it quietly destroys the KV-reuse and
   content-hash wins that made phenotyper load-bearing. Node-aligned
   drops also hand the prompt-cache splice rung segment-aligned
   degradation for free.

**The strict-determinism guarantee (binding).** Phenotyper must remain
usable in a STRICTLY DETERMINISTIC mode — code generation being the
canonical consumer: same source, same inputs, same bytes, always.
Therefore: annotations are INERT unless the caller invokes the
budgeted renderer (a phenotype carrying elasticity renders via the
plain `render()` exactly as it does today, byte-identical); the
budgeted path is itself a pure function of (content, budget, measure);
and nothing — no clock, no environment, no iteration-order hazard —
may enter either path. Elasticity adds a second DETERMINISTIC
renderer, never a nondeterministic one.

Non-goals, to keep the crate's shape: no tokenizer dependency, no
runtime template engine (build-time codegen stays), no LLM/provider
awareness. The consumer brings the intelligence; phenotyper guarantees
structure, both ways.
