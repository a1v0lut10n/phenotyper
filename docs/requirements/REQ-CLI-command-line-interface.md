# REQ-CLI — Command-Line Interface Requirements

## Status
v1 — Derived from `phenotyper_v1_language_and_compiler_spec.md` section 23.

---

## REQ-CLI-001 — Binary name

The compiler binary shall be named `phenotyper`.

---

## REQ-CLI-002 — Required v1 subcommands

| Subcommand | Description | Example |
|------------|-------------|---------|
| `check` | Validate a source file without generating code | `phenotyper check example.pht` |
| `build` | Validate and generate Rust code | `phenotyper build example.pht --out src/generated` |

---

## REQ-CLI-003 — Optional v1 subcommands

| Subcommand | Description | Example |
|------------|-------------|---------|
| `dump-ast` | Print the parsed AST for debugging | `phenotyper dump-ast example.pht` |
| `dump-ir` | Print the normalized IR for debugging | `phenotyper dump-ir example.pht` |

---

## REQ-CLI-004 — Flags

| Flag | Applicable to | Description |
|------|---------------|-------------|
| `--out <path>` | `build` | Output directory for generated Rust code |
| `--json` | `check` | Emit diagnostics as JSON |

---

## REQ-CLI-005 — Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Validation or generation failure |
| `2` | Usage/argument error |

---

## REQ-CLI-006 — Source file handling

- The compiler shall accept both `.pht` and `.md` files.
- For `.md` files, `pht` fenced code blocks are extracted automatically.
- File type is determined by extension.

---

## REQ-CLI-007 — Diagnostic output

- By default, diagnostics are printed in human-readable format to stderr.
- When `--json` is specified, diagnostics are emitted as a JSON array to stdout.
- Diagnostics include file path, line, column, severity, message, and optional fix suggestions.
