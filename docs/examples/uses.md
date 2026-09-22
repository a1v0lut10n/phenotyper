# Example: importing a namespace with `uses`

Two namespaces, one search root (PHT-0027, REQ-LANG-003). The shared
declaration lives in its own namespace:

`<root>/aivolution/core/badge.pht`

```pht
aivolution/core/badge:
Badge plural Badges:
    label: required string,
    @(label)
;
.
```

The importer brings it into unqualified scope with `uses`:

`<root>/aivolution/ai/card.pht`

```pht
aivolution/ai/card:
uses aivolution/core/badge;
Card:
    title: required string,
    badge: required Badge,
    @(title), " [", @(badge), "]"
;
.
```

Compile the set — from `build.rs`:

```rust
phenotyper::compile_with_roots("phenotypes/aivolution/ai/card.pht",
                               &["phenotypes"], out_dir)?;
```

or from the CLI:

```bash
phenotyper build phenotypes/aivolution/ai/card.pht --out generated/ --root phenotypes
```

Both namespaces are compiled once, in dependency order, into a mountable
module tree:

```
generated/mod.rs                          // pub mod aivolution;
generated/aivolution/mod.rs               // pub mod ai; pub mod core;
generated/aivolution/ai/card/mod.rs       // Card, CardBuilder, …
generated/aivolution/core/badge/mod.rs    // Badge, Badges, …
```

The generated `Card` references `Badge` by relative Rust path
(`super::super::core::badge::Badge`) — nothing is re-emitted — so the
tree mounts anywhere with a single `pub mod aivolution;` (or an
`include!` of `generated/mod.rs`'s parent module). Rendering crosses the
boundary through the exporter's own `Render` trait:

```rust
use generated::aivolution::{ai::card, core::badge};

let card = card::Card {
    title: "Genite".into(),
    badge: badge::Badge { label: "gold".into() },
};
assert_eq!(card::Render::render(&card), "Genite [gold]");
```

Scope rules, briefly: local declarations shadow imports (the compiler
warns, naming the exporter); a name importable from two `uses`
namespaces is a hard error at its use site (declare it locally or drop
one import); `uses` cycles are hard errors naming the cycle.
