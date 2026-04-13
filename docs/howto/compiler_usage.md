# Compiler Usage

This guide covers the `phenotyper` command-line compiler, including
all subcommands, flags, and exit codes.

## Installation

Build from source:

```bash
cargo install --path crates/phenotyper-cli
```

Or build the workspace:

```bash
cargo build --release
# Binary: target/release/phenotyper
```

## Subcommands

### `check` — Validate a Source File

Run the full pipeline (parse → symbols → IR → semantic validation)
without generating code:

```bash
phenotyper check path/to/file.pht
phenotyper check path/to/file.md
```

Output on success:

```
✓ path/to/file.pht — no errors
```

### `build` — Compile and Generate Rust

Run the full pipeline and generate Rust code to the output directory:

```bash
phenotyper build path/to/file.pht --out generated/
phenotyper build path/to/file.md -o generated/
```

Output on success:

```
✓ path/to/file.pht → generated/ — no errors
```

The output directory structure mirrors the namespace:

```
generated/
  aivolution/
    format/
      csv/
        mod.rs      ← generated Rust module
```

> **Note:** Nested phenotype types are flattened into the same `mod.rs`
> as their parent.

### `dump-ast` — Print the AST

Print the parsed AST for debugging:

```bash
phenotyper dump-ast path/to/file.pht
```

Output is Rust `Debug` format of the AST.

### `dump-ir` — Print the IR

Print the normalized intermediate representation:

```bash
phenotyper dump-ir path/to/file.pht
```

Output is Rust `Debug` format of the `PhenotypeModule`. Nested types
appear as flattened entries with `parent_context` set for types that
reference parent fields.

## Flags

### `--json` — Machine-Readable Diagnostics

Available on `check` and `build`. Outputs diagnostics as NDJSON
(newline-delimited JSON) to **stdout**:

```bash
phenotyper check --json file.pht
```

Each diagnostic is a JSON object:

```json
{"severity":"error","file":"file.pht","line":5,"col":0,"summary":"unknown type `Foo`","explanation":"...","suggestion":"..."}
```

Without `--json`, diagnostics are printed as human-readable text
to **stderr**.

## File Types

The compiler accepts two file extensions:

| Extension | Behavior |
|-----------|----------|
| `.pht` | Parsed directly as the core language |
| `.md` | Extracts ` ```pht ` fenced blocks, concatenates, then parses |

Any other extension produces an error:

```
error: unsupported file extension '.txt' — expected .pht or .md
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success — no errors |
| `1` | Failure — compilation errors |
| `2` | Usage error — invalid arguments (handled by clap) |

## Examples

### Check all files in a directory

```bash
for f in src/**/*.pht; do
    phenotyper check "$f"
done
```

### CI pipeline validation

```bash
phenotyper check --json src/phenotypes/main.pht > diagnostics.json
if [ $? -ne 0 ]; then
    echo "Phenotyper compilation failed"
    exit 1
fi
```

### Generate code and format

```bash
phenotyper build src/phenotypes/main.pht --out src/generated/
# Generated code is already rustfmt'd if rustfmt is available
```
