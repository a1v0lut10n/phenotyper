# Build Script Integration

This guide explains how to integrate Phenotyper code generation into
a Rust project's `build.rs`, similar to how `prost-build` or `rustemo`
work.

## Overview

The `phenotyper-core` crate provides a high-level `compile()` API
that can be called from a `build.rs` script. This automates code
generation as part of `cargo build`, so generated Rust code is always
up to date.

## Setup

### 1. Add `phenotyper-core` as a Build Dependency

In your project's `Cargo.toml`:

```toml
[build-dependencies]
phenotyper-core = { path = "../path/to/phenotyper/crates/phenotyper-core" }
```

Or, once published:

```toml
[build-dependencies]
phenotyper-core = "0.2"
```

### 2. Create a `build.rs`

```rust
fn main() {
    // Compile a .pht file and write output to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();

    phenotyper_core::compile(
        "src/phenotypes/csv.pht",     // Source file
        &out_dir,                      // Output directory
    ).expect("phenotyper compilation failed");
}
```

### 3. Include the Generated Code

In your `lib.rs` or `main.rs`:

```rust
// Include the generated module
include!(concat!(env!("OUT_DIR"), "/aivolution/format/csv/mod.rs"));
```

Or, if you prefer a named module:

```rust
mod generated {
    include!(concat!(env!("OUT_DIR"), "/aivolution/format/csv/mod.rs"));
}
```

## API Reference

### `phenotyper_core::compile(source_path, out_dir)`

File-based entry point. Reads the source file, runs the full pipeline,
and writes generated Rust to the output directory.

```rust
pub fn compile(
    source_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> CompileResult
```

**Features:**
- Automatically detects `.pht` vs `.md` by file extension
- Emits `cargo:rerun-if-changed={source_path}` so cargo only
  rebuilds when the source file changes
- Returns `Ok(CompileOutput)` on success or `Err(Vec<Diagnostic>)`
  on failure

### `phenotyper_core::compile_source(source, file_name, out_dir)`

String-based entry point for embedding or testing. Does not read
from the filesystem.

```rust
pub fn compile_source(
    source: &str,
    file_name: &str,
    out_dir: Option<&Path>,
) -> CompileResult
```

**Parameters:**
- `source` — The phenotyper source code as a string
- `file_name` — A display name for diagnostics (e.g., `"main.pht"`)
- `out_dir` — If `Some`, writes generated code; if `None`, only
  compiles and returns the code without writing

### `CompileOutput`

```rust
pub struct CompileOutput {
    pub code: String,         // Generated Rust source
    pub namespace: String,    // DSL namespace (e.g., "aivolution/format/csv")
    pub warnings: Vec<Diagnostic>,
}
```

## Complete Example

### Project Structure

```
my-project/
├── Cargo.toml
├── build.rs
├── src/
│   ├── main.rs
│   └── phenotypes/
│       └── csv.pht
```

### `Cargo.toml`

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[build-dependencies]
phenotyper-core = { path = "../phenotyper/crates/phenotyper-core" }
```

### `build.rs`

```rust
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile all .pht files in the phenotypes directory
    let phenotypes_dir = Path::new("src/phenotypes");

    for entry in std::fs::read_dir(phenotypes_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "pht" || ext == "md") {
            phenotyper_core::compile(&path, &out_dir)
                .unwrap_or_else(|errors| {
                    for e in &errors {
                        eprintln!(
                            "cargo:warning=phenotyper: {}:{}: {}",
                            e.file, e.line, e.summary
                        );
                    }
                    panic!("phenotyper compilation failed for {}", path.display());
                });
        }
    }
}
```

### `src/main.rs`

```rust
// Include the generated CSV module
include!(concat!(env!("OUT_DIR"), "/aivolution/format/csv/mod.rs"));

fn main() {
    let field = CsvFieldValue {
        value: ScalarValue::String("hello".to_string()),
    };

    println!("{}", field.render());
}
```

## Tips

1. **Use `OUT_DIR`** — Always write to `OUT_DIR` in `build.rs` to
   follow Cargo conventions
2. **Error handling** — The `compile()` API returns diagnostics as
   `Vec<Diagnostic>` which you can format for build output
3. **Multiple files** — Call `compile()` once per source file; each
   generates its own namespace directory
4. **Rebuild tracking** — `compile()` automatically emits
   `cargo:rerun-if-changed` so builds are incremental
5. **Markdown sources** — `.md` files work identically; the API
   auto-detects the format
