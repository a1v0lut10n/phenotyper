# REQ-TEST — Testing Strategy Requirements

## Status
v1 — Derived from `phenotyper_v1_language_and_compiler_spec.md` section 24.

---

## REQ-TEST-001 — Parser tests

| Category | Description |
|----------|-------------|
| Golden tests | Valid syntax files with expected AST snapshots |
| Negative tests | Malformed syntax that must produce specific parse errors |
| Comment handling | Correct treatment of line and block comments |
| Whitespace | Insensitivity to whitespace variations |
| Singular/plural headers | Parsing both forms correctly |
| Namespace and uses | Correct parsing of `namespace` and `uses` declarations |
| Enum declarations | `type Name: [member1, member2]` parsing |
| Type aliases | `type Name: TypeExpr` parsing |
| Markdown extraction | `pht` block extraction from `.md` files with correct source mapping |

---

## REQ-TEST-002 — Semantic validation tests

| Category | Expected result |
|----------|-----------------|
| Duplicate singular type names | Error |
| Duplicate plural type names | Error |
| Singular/plural name collisions | Error |
| Duplicate field names within a type | Error |
| Unknown type references | Error |
| Unknown field references | Error |
| Invalid directive arity | Error |
| Direct `@emit` of plural fields | Error, suggest `@join` |
| Invalid separator types | Error |
| Direct `@emit` of optional fields | Error |
| Cyclic type graphs | Error |
| Shadowing via `uses` of local name | Warning |
| Duplicate enum member names | Error |

---

## REQ-TEST-003 — Code generation tests

| Category | Description |
|----------|-------------|
| Snapshot tests | Generated Rust code compared against golden files |
| Compile tests | Generated code must compile successfully |
| Runtime rendering tests | Constructed instances produce expected textual output |
| Naming tests | Singular/plural Rust output names are correct |
| Builder tests | Builder API enforces required fields, rejects missing values |
| Union tests | Correct enum generation and rendering |
| Enum tests | Correct Rust enum generation from `type Name: [...]` |

---

## REQ-TEST-004 — End-to-end tests

At minimum, v1 shall include:

| Test case | Exercises |
|-----------|-----------|
| CSV example | Singular/plural types, unions, `@join`, recursive rendering, separators |
| Structured prompt example | Composition, string literals, `@emit`, multi-field render bodies |
| Union-heavy example | Multiple union-typed fields, enum generation, variant rendering |
| Markdown source | End-to-end from `.md` extraction through to code generation |

---

## REQ-TEST-005 — Diagnostic tests

- Each semantic error category shall have at least one test verifying:
  - The correct error is produced.
  - The diagnostic includes accurate source location (line, column).
  - The diagnostic message is actionable.
  - Fix suggestions appear where applicable.

---

## REQ-TEST-006 — Test fixture organization

Test fixtures should be organized by category:

```
tests/
├── fixtures/
│   ├── valid/          # Valid .pht and .md files
│   ├── invalid/        # Files expected to produce errors
│   └── golden/         # Expected output snapshots
├── parser/             # Parser unit tests
├── semantic/           # Semantic validation tests
├── codegen/            # Code generation tests
└── e2e/                # End-to-end integration tests
```
