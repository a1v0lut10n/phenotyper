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

Non-goals, to keep the crate's shape: no tokenizer dependency, no
runtime template engine (build-time codegen stays), no LLM/provider
awareness. The consumer brings the intelligence; phenotyper guarantees
structure, both ways.
