# M13 — Nested Phenotype Declarations

## Phase 1: Grammar & Parser
- [x] T-250: Grammar — NestedType body item + FieldPath (left-factored for LALR(1))
- [x] T-251: Actions — NestedType + FieldPath AST nodes
- [x] T-252: Parser unit tests (5 tests)

## Phase 2: AST
- [x] T-253: AST types (NestedType/NestedTypePlural wrapping TypeDef, FieldPath)
- [x] T-254: AST unit tests (covered by parser tests)

## Phase 3: Symbol Table
- [x] T-255: collect.rs — walk nested types
- [x] T-256: resolve.rs — scoped field paths (last-segment resolution)
- [x] T-257: Symbol table tests (covered by semantic tests)

## Phase 4: IR & Semantic
- [x] T-258: IR lowering — flatten + extra_types collection
- [x] T-259: IR types — parent_context (deferred to Phase 5)
- [x] T-260: Semantic validation (recursive via existing)
- [x] T-261: Semantic tests (2 tests) + IR tests (3 tests)

## Phase 5: Code Generation
- [x] T-262: render_with_parent
- [x] T-263: Parent render calls
- [x] T-264: Naming (uses existing naming module)
- [x] T-265: Codegen tests (covered by existing codegen tests)

## Phase 6: Integration
- [x] T-266: nested_basic.pht fixture
- [ ] T-267: javaclass.pht fixture
- [ ] T-268: e2e tests
- [ ] T-269: CLI tests
