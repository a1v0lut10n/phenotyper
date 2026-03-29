# Phenotyper Design Memo

## Status
Draft for concept refinement and v1 planning.

## Executive Summary

Phenotyper is a domain-specific language and compiler for defining the shape of structured textual artifacts and generating tooling that can construct, render, validate, and eventually parse them. It is intended for cases where outputs must remain human-readable while also being structurally constrained enough for reliable programmatic generation, verification, and round-tripping.

The core motivation is that many important outputs in modern systems are neither free-form prose nor simple data objects. They are artifacts such as prompts, reports, configuration files, source-code fragments, tables, markup, and semi-formal text interfaces. Existing tools only partially address this space: template engines are good at substitution, schema languages are good at data modeling, and parser generators are good at recognition. Phenotyper aims to bridge the gap by making artifact structure itself the first-class concern.

Phenotyper should be positioned as a **structural artifact-definition system** rather than merely as a template language. A phenotype definition should describe a family of valid artifacts in a notation that remains understandable to humans while being precise enough for machines. From that definition, Phenotyper should generate typed construction APIs, renderers, validators, and later parsers.

A future enhancement with strong strategic value is an **import and conversion CLI** that can ingest templates written in common ecosystems such as Jinja2, Django templates, Mako, Handlebars, Razor, and EJS, then convert them into candidate phenotype definitions. This would give Phenotyper a migration path from existing template-based systems and position it not only as a greenfield authoring language, but also as an interoperability and modernization tool.

## Problem Statement

Many software systems need to produce artifacts that follow recognizable structural rules while remaining readable to humans. These artifacts often need to satisfy several properties simultaneously:

- they must be generated reproducibly,
- they must conform to a recognizable layout,
- they should be inspectable and editable by humans,
- they should ideally be parsable back into structured representations,
- and in AI-assisted workflows they should be stable across models, versions, and execution contexts.

Traditional template systems solve variable insertion and control-flow interpolation, but they generally do not give a strong structural model of the resulting artifact. Schema systems describe data, but not how that data manifests as concrete textual form. Parser generators recognize textual structure, but do not typically provide a pleasant authoring model for defining artifact families or generated construction APIs.

Phenotyper exists to fill this gap by treating structured textual artifacts as first-class objects with declarative definitions.

## Product Vision

Phenotyper should allow a developer to define a structured artifact once and then derive from that definition:

- typed APIs for constructing valid instances,
- renderers that emit concrete artifacts,
- validators that check structural conformance,
- documentation of artifact structure,
- and later parsers that reify existing artifacts back into typed representations.

The long-term value proposition is simple:

> define the shape of an artifact once, then generate the tooling needed to create and interpret it correctly.

This is especially relevant for LLM-assisted systems, where output structure often matters as much as output content.

## Terminology

The project benefits from a small and precise vocabulary.

### Phenotyper
The language, compiler, and tooling ecosystem.

### Phenotype definition
A source file written in the Phenotyper DSL that defines one or more artifact families or types.

### Phenotype type
A named artifact kind such as `CSVRecord`, `CSVLine`, or `CSVFile`.

### Phenotype instance
A concrete rendered artifact that conforms to a phenotype definition.

### Renderer
Generated or runtime code that turns a structured phenotype instance into its concrete textual form.

### Parser
Generated or runtime code that reads a concrete artifact and reconstructs a structured representation.

### Directive
A declarative rendering instruction embedded in a phenotype definition, such as interpolation or joining.

### Builder API
Generated typed APIs that guide client code toward constructing conformant artifact instances.

## Conceptual Model

Phenotyper defines **families of valid artifacts**. A phenotype definition can describe:

- named artifact types,
- fields and their types,
- cardinality such as singular or plural elements,
- composition relationships among artifact types,
- rendering directives,
- and later parsing or round-tripping behavior.

A concrete example is the uploaded CSV phenotype, which defines `CSVRecord`, `CSVLine`, and `CSVFile` as compositional artifact types. It models required typed fields, plurality, and explicit rendering behavior through directives such as `@(field)` and `@join(records, separator)`. This example is useful because it shows that Phenotyper is describing structure and manifestation together, rather than just placeholder substitution.

## Design Principles

### Human-readable first
The DSL should remain understandable to humans without forcing readers to reconstruct an invisible schema from code.

### Structural precision
The language must be constrained enough to support conformance checks, typed builders, and predictable output generation.

### Declarative over imperative
Phenotyper should describe what a valid artifact looks like, not become a general-purpose programming language.

### Single source of truth
A phenotype definition should serve as the canonical source for all generated tooling.

### Round-tripping where practical
A strong long-term property is that artifact definitions support both rendering and parsing.

### Interoperability matters
Phenotyper should not assume a greenfield world. It should provide a credible migration path from existing template ecosystems.

## Architecture Overview

A useful architecture model has three layers.

### 1. Definition layer
The Phenotyper DSL defines artifact families, fields, composition rules, constraints, and directives.

### 2. Compilation layer
The compiler parses the DSL and transforms it into an intermediate representation suitable for analysis and code generation.

### 3. Generated tooling layer
Code generators derive builders, renderers, validators, documentation, and later parsers from the intermediate representation.

This layered approach keeps the system conceptually clean and creates room for multiple code-generation targets over time.

## Likely Implementation Direction

The current direction suggests a Rust-based implementation using Rustemo for parsing. The first generation backend can be pragmatic and use a conventional template engine internally, while the language and artifact model remain the core innovation. Over time, the system can move toward self-hosting and self-bootstrapping.

A reasonable internal pipeline looks like this:

1. parse DSL source into an AST,
2. normalize the AST into an intermediate representation,
3. run semantic validation over types, fields, plurality, and directives,
4. generate target-specific tooling,
5. optionally support artifact parsing and round-tripping later.

## v1 Scope

The first version should remain deliberately focused.

### In scope

- a human-readable punctuation-driven DSL,
- named artifact types,
- typed fields and requiredness,
- plurality and composition,
- a limited set of rendering directives,
- Rust-based parser implementation,
- generated Rust builder APIs,
- rendering and structural validation,
- at least one concrete end-to-end example such as CSV or structured prompts.

### Out of scope for v1

- arbitrary control flow in the DSL,
- general-purpose scripting semantics,
- full bidirectional parsing for all artifact forms,
- broad multi-language code generation from day one,
- deep semantic validation beyond structural conformance,
- automatic template-import conversion as a production-quality feature.

## Future Enhancement: Template Conversion CLI

### Summary

A valuable future enhancement is a CLI that imports templates from popular ecosystems and converts them into candidate phenotype definitions.

Illustrative target ecosystems include:

- Python: Jinja2, Django templates, Mako,
- Rust: Handlebars,
- .NET: Razor,
- JavaScript/TypeScript: EJS,
- and potentially others later.

### Why this matters

This capability would expand Phenotyper beyond greenfield adoption.

It would enable:

- migration of legacy template systems into phenotype definitions,
- comparative analysis between template logic and phenotype structure,
- faster onboarding for teams that already have extensive template inventories,
- a practical bridge from imperative or interpolation-heavy templates toward more declarative artifact definitions,
- and a path for organizations to standardize artifact generation while preserving prior investments.

Strategically, this could turn Phenotyper into both an authoring system and a modernization tool.

### Recommended positioning

This should be treated as an **interoperability and migration feature**, not part of the core language definition for v1.

That distinction matters because imported templates will often contain behavior that is richer, more imperative, and more runtime-dependent than the initial Phenotyper language should allow. The conversion process should therefore be framed as producing:

- candidate phenotype definitions,
- compatibility annotations,
- and migration warnings where exact semantic preservation is not possible.

### Conversion model

A sensible long-term pipeline would be:

1. parse the source template into a source-template AST,
2. normalize it into a language-neutral template IR,
3. identify artifact structure, placeholders, loops, conditionals, partials, includes, and filters/helpers,
4. map the recognized structural subset into a Phenotyper IR,
5. emit a phenotype definition plus a report of unresolved or lossy constructs.

### Expected challenges

This feature is powerful, but difficult.

Key difficulties include:

- different template languages mix structure and control flow in incompatible ways,
- helpers, filters, and custom functions may not have direct Phenotyper equivalents,
- some templates are only understandable with runtime context,
- whitespace and escaping semantics differ significantly across ecosystems,
- layout intent is sometimes implicit and only emerges at render time,
- template inheritance and includes may need separate Phenotyper concepts or import-time flattening.

### Practical design stance

The initial goal should not be perfect one-click equivalence.

A realistic design goal is:

> infer the structural intent of an existing template and convert the subset that maps cleanly into a phenotype definition, while surfacing the remainder as explicit migration issues.

This would still be extremely valuable.

### Suggested future CLI behavior

Potential commands might look like:

```bash
phenotyper import-template --from jinja2 invoice_template.j2 --out invoice.pht
phenotyper import-template --from handlebars component.hbs --out component.pht
phenotyper analyze-template --from razor EmailTemplate.cshtml
```

Potential outputs could include:

- generated `.pht` files,
- a conversion report,
- warnings about unsupported constructs,
- optional sidecar files capturing original helpers or assumptions.

### Relationship to core product strategy

This enhancement strengthens several parts of the overall product story:

- it lowers adoption friction,
- it encourages incremental migration,
- it makes Phenotyper legible to teams already invested in templates,
- and it provides real-world corpora that can inform evolution of the language.

## Risks

Several risks should remain visible.

### Language overgrowth
The DSL could become too expressive and drift toward a programming language.

### Underconstrained semantics
If directives or typing rules are too vague, generated tooling may not provide meaningful guarantees.

### Parser complexity
A punctuation-heavy language requires crisp grammar and strong tooling discipline.

### Feature pressure from migration
Imported-template use cases may create pressure to mirror template-language control flow too early.

### Round-tripping difficulty
Supporting reliable parsing for arbitrary textual artifacts will be harder than builder generation.

## Open Design Questions

Several questions should be answered before or during v1 work.

1. What is the exact boundary between structure description and generation logic?
2. Which directives are built into the core language?
3. How should plurality and optionality be represented syntactically?
4. What should the intermediate representation contain?
5. How much validation belongs in the compiler versus generated code?
6. What is the exact generated Rust API shape?
7. Which subset of parsing should be targeted first?
8. Should imported-template constructs have a formal compatibility taxonomy from the start?
9. How should unsupported imported-template features be reported to users?

## Success Criteria

A successful first milestone would demonstrate that Phenotyper can:

- define a small but expressive family of artifacts,
- compile those definitions reliably,
- generate usable Rust builders and renderers,
- validate structural conformance,
- and support at least one convincing real-world use case such as CSV or structured prompt formats.

A successful later milestone for template conversion would demonstrate that the CLI can ingest a representative subset of mainstream templates and produce useful, human-reviewable phenotype drafts with clear migration diagnostics.

## Recommended Next Step

The next design document should be a **v1 language and compiler specification**. That memo should define:

- surface syntax,
- grammar and parsing strategy,
- AST and intermediate representation,
- directive semantics,
- type system and cardinality model,
- validation rules,
- generated Rust API conventions,
- and explicit extension points for later parser generation and template-import conversion.

## Conclusion

Phenotyper should be treated as a structural artifact-definition system: a DSL and compiler for describing families of renderable, human-readable artifacts in a way that supports reliable generation and, eventually, parsing. Its differentiator is that it models artifact shape directly while preserving readability and enabling generated tooling.

The proposed template-conversion CLI is a strong future addition because it gives the project a migration and interoperability story. Incorporated carefully, it can broaden adoption without distracting from the narrower and more disciplined goals of v1.
