// SPDX-License-Identifier: Apache-2.0
//! End-to-end integration tests — validate the full pipeline from `.pht` source
//! through code generation and verify the generated code structure.

use phenotyper::diagnostic;

/// Helper: compile a .pht source and return the generated Rust code.
fn compile_ok(source: &str) -> String {
    match phenotyper::compile_source(source, "test.pht", None) {
        Ok(output) => {
            assert!(
                output.warnings.is_empty(),
                "unexpected warnings: {:?}",
                output.warnings
            );
            output.code
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("{}", diagnostic::format_human(e));
            }
            panic!("compilation failed with {} error(s)", errors.len());
        }
    }
}

/// Helper: compile and expect errors.
fn compile_err(source: &str) -> Vec<phenotyper::diagnostic::Diagnostic> {
    match phenotyper::compile_source(source, "test.pht", None) {
        Ok(_) => panic!("expected compilation to fail"),
        Err(errors) => errors,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T-113: CSV end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_csv_fixture_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let code = compile_ok(source);

    // Core structures
    assert!(
        code.contains("pub struct CsvFieldValue"),
        "missing CsvFieldValue struct"
    );
    assert!(
        code.contains("pub struct CsvFieldValues"),
        "missing CsvFieldValues struct"
    );
    assert!(
        code.contains("pub struct CsvLine"),
        "missing CsvLine struct"
    );
    assert!(
        code.contains("pub struct CsvLines"),
        "missing CsvLines struct"
    );
    assert!(
        code.contains("pub struct CsvFile"),
        "missing CsvFile struct"
    );

    // Union type
    assert!(
        code.contains("pub enum ScalarValue"),
        "missing ScalarValue enum"
    );
    assert!(code.contains("Int64(i64)"), "missing Int64 variant");
    assert!(code.contains("String(String)"), "missing String variant");

    // Builders
    assert!(
        code.contains("pub struct CsvFieldValueBuilder"),
        "missing builder"
    );
    assert!(
        code.contains("pub struct CsvLineBuilder"),
        "missing builder"
    );
    assert!(
        code.contains("pub struct CsvFileBuilder"),
        "missing builder"
    );

    // Plural builders
    assert!(
        code.contains("pub struct CsvFieldValuesBuilder"),
        "missing plural builder"
    );
    assert!(
        code.contains("pub struct CsvLinesBuilder"),
        "missing plural builder"
    );

    // Render implementations
    assert!(
        code.contains("impl Render for CsvFieldValue"),
        "missing Render impl"
    );
    assert!(
        code.contains("impl Render for CsvLine"),
        "missing Render impl"
    );
    assert!(
        code.contains("impl Render for CsvFile"),
        "missing Render impl"
    );
    assert!(
        code.contains("impl Render for CsvFieldValues"),
        "missing plural Render impl"
    );
    assert!(
        code.contains("impl Render for CsvLines"),
        "missing plural Render impl"
    );

    // Render trait
    assert!(code.contains("pub trait Render"), "missing Render trait");
    assert!(
        code.contains("fn render_into(&self, out: &mut String)"),
        "missing render_into"
    );

    // BuildError
    assert!(code.contains("pub enum BuildError"), "missing BuildError");
    assert!(
        code.contains("MissingField"),
        "missing MissingField variant"
    );
}

#[test]
fn e2e_csv_namespace() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let output = phenotyper::compile_source(source, "test.pht", None).unwrap();
    assert_eq!(output.namespace, "aivolution/format/csv");
}

#[test]
fn e2e_csv_join_separator() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let code = compile_ok(source);

    // CsvLine uses @join(fields, separator) — field-based separator
    assert!(
        code.contains("out.push_str(&self.separator)"),
        "missing field separator in join"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T-114: Markdown container end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_md_container_compiles() {
    // Compile the csv.md example (markdown with embedded ```pht blocks)
    let md_source = include_str!("../../../docs/examples/csv.md");
    let output = phenotyper::compile_source(md_source, "csv.md", None).unwrap();

    assert_eq!(output.namespace, "aivolution/format/csv");
    assert!(output.code.contains("pub struct CsvFieldValue"));
    assert!(output.code.contains("pub struct CsvLine"));
    assert!(output.code.contains("pub struct CsvFile"));
    assert!(output.code.contains("pub enum ScalarValue"));
    assert!(output.code.contains("pub trait Render"));
}

#[test]
fn e2e_md_all_examples_compile() {
    let examples = [
        ("csv.md", include_str!("../../../docs/examples/csv.md")),
        (
            "prompt.md",
            include_str!("../../../docs/examples/prompt.md"),
        ),
        (
            "config.md",
            include_str!("../../../docs/examples/config.md"),
        ),
        (
            "report.md",
            include_str!("../../../docs/examples/report.md"),
        ),
    ];

    for (name, source) in &examples {
        let result = phenotyper::compile_source(source, name, None);
        assert!(result.is_ok(), "{name} failed: {:?}", result.err());
    }
}

#[test]
fn e2e_md_no_pht_blocks_error() {
    // A markdown file with no ```pht blocks should produce an error
    let md = "# Just a regular markdown file\n\nNo phenotyper here.\n";
    let result = phenotyper::compile_source(md, "empty.md", None);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.summary.contains("no")));
}

// ═══════════════════════════════════════════════════════════════════════════
// T-115: Structured prompt end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_prompt_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/prompt.pht");
    let code = compile_ok(source);

    // Structs
    assert!(
        code.contains("pub struct PromptSection"),
        "missing PromptSection"
    );
    assert!(
        code.contains("pub struct PromptSections"),
        "missing PromptSections"
    );
    assert!(
        code.contains("pub struct SystemPrompt"),
        "missing SystemPrompt"
    );

    // Optional field
    assert!(
        code.contains("pub context: Option<String>"),
        "missing optional context"
    );

    // Text literals in render
    assert!(code.contains("# System Prompt"), "missing title literal");
    assert!(code.contains("You are "), "missing role prefix");

    // @ifset for optional context
    assert!(
        code.contains("if let Some(ref val) = self.context"),
        "missing @ifset codegen"
    );
}

#[test]
fn e2e_prompt_namespace() {
    let source = include_str!("../../../tests/fixtures/valid/prompt.pht");
    let output = phenotyper::compile_source(source, "test.pht", None).unwrap();
    assert_eq!(output.namespace, "aivolution/ai/prompt");
}

// ═══════════════════════════════════════════════════════════════════════════
// T-116: Union-heavy (config) end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_config_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/config.pht");
    let code = compile_ok(source);

    // Enum type
    assert!(
        code.contains("pub enum Visibility"),
        "missing Visibility enum"
    );
    assert!(code.contains("Public,"), "missing Public variant");
    assert!(code.contains("Private,"), "missing Private variant");
    assert!(code.contains("Internal,"), "missing Internal variant");

    // Enum renders original spelling
    assert!(
        code.contains("out.push_str(\"public\")"),
        "missing original-spelling render"
    );

    // Union type alias
    assert!(
        code.contains("pub enum ConfigValue"),
        "missing ConfigValue union"
    );
    assert!(code.contains("Int64(i64)"), "missing Int64 in union");
    assert!(code.contains("Bool(bool)"), "missing Bool in union");

    // Nested structures
    assert!(
        code.contains("pub struct ConfigEntry"),
        "missing ConfigEntry"
    );
    assert!(
        code.contains("pub struct ConfigEntries"),
        "missing ConfigEntries"
    );
    assert!(
        code.contains("pub struct ConfigSection"),
        "missing ConfigSection"
    );
    assert!(
        code.contains("pub struct ConfigSections"),
        "missing ConfigSections"
    );
    assert!(code.contains("pub struct ConfigFile"), "missing ConfigFile");

    // Text literals
    assert!(
        code.contains("out.push_str(\" = \")"),
        "missing ' = ' separator"
    );
    assert!(code.contains("out.push_str(\" [\")"), "missing ' [' prefix");
}

// ═══════════════════════════════════════════════════════════════════════════
// T-118: Optional-field (report) end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_report_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/report.pht");
    let code = compile_ok(source);

    // Structs
    assert!(code.contains("pub struct Report"), "missing Report");
    assert!(code.contains("pub struct ReportLine"), "missing ReportLine");
    assert!(code.contains("pub struct Tag"), "missing Tag");

    // Optional fields
    assert!(
        code.contains("pub subtitle: Option<String>"),
        "missing optional subtitle"
    );
    assert!(
        code.contains("pub footer: Option<String>"),
        "missing optional footer"
    );

    // Collection field (plural wrapper type)
    assert!(code.contains("pub tags: Tags"), "missing tags field");

    // @ifset for subtitle
    assert!(
        code.contains("if let Some(ref val) = self.subtitle"),
        "missing @ifset for subtitle"
    );

    // @ifset for footer
    assert!(
        code.contains("if let Some(ref val) = self.footer"),
        "missing @ifset for footer"
    );

    // @ifnotempty for tags (plural wrapper uses as_slice().is_empty())
    assert!(
        code.contains("if !self.tags.as_slice().is_empty()"),
        "missing @ifnotempty for tags"
    );
}

#[test]
fn e2e_report_join_tags() {
    let source = include_str!("../../../tests/fixtures/valid/report.pht");
    let code = compile_ok(source);

    // Tags joined with ", "
    assert!(
        code.contains("out.push_str(\", \")"),
        "missing tag join separator"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T-108: CLI integration — error handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_compile_error_on_invalid_source() {
    let errors = compile_err("this is not valid phenotyper source");
    assert!(!errors.is_empty(), "expected parse errors");
}

#[test]
fn e2e_compile_error_on_missing_required_type() {
    let errors = compile_err(
        r#"
        test/types:
        Record:
            name: required UnknownType,
            @(name)
        ;
    .
    "#,
    );
    assert!(!errors.is_empty(), "expected symbol errors");
    assert!(
        errors.iter().any(|e| e.summary.contains("unknown type")),
        "expected 'unknown type' in error"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// File output (T-113 complement)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_compile_writes_output() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let out_dir = std::env::temp_dir().join("phenotyper-e2e-test");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = phenotyper::compile_source(source, "test.pht", Some(&out_dir)).unwrap();

    let expected_file = out_dir.join("aivolution/format/csv/mod.rs");
    assert!(expected_file.exists(), "output file not created");

    let written = std::fs::read_to_string(&expected_file).unwrap();
    assert_eq!(
        written, output.code,
        "written file doesn't match returned code"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// Generated code quality checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_generated_code_has_header() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let code = compile_ok(source);
    assert!(
        code.starts_with("// This file is @generated"),
        "missing @generated header"
    );
}

#[test]
fn e2e_generated_code_has_allow_attributes() {
    let source = include_str!("../../../tests/fixtures/valid/csv_basic.pht");
    let code = compile_ok(source);
    assert!(
        code.contains("#![allow(dead_code"),
        "missing #![allow] attribute"
    );
}

#[test]
fn e2e_all_fixtures_compile() {
    let fixtures = [
        include_str!("../../../tests/fixtures/valid/csv_basic.pht"),
        include_str!("../../../tests/fixtures/valid/prompt.pht"),
        include_str!("../../../tests/fixtures/valid/config.pht"),
        include_str!("../../../tests/fixtures/valid/report.pht"),
    ];

    for (i, source) in fixtures.iter().enumerate() {
        let result = phenotyper::compile_source(source, &format!("fixture_{i}.pht"), None);
        assert!(result.is_ok(), "fixture {i} failed: {:?}", result.err());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T-095: Codegen compile tests — verify generated Rust compiles with rustc
// ═══════════════════════════════════════════════════════════════════════════

/// Helper: compile generated Rust code with `rustc` and check it passes.
fn assert_rustc_compiles(code: &str, label: &str) {
    use std::io::Write;

    let tmp_dir = std::env::temp_dir().join(format!("phenotyper-compile-test-{label}"));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let src_file = tmp_dir.join("generated.rs");

    // The generated code already has #![allow(...)] — use it as-is with a main()
    let wrapped = format!("{code}\nfn main() {{}}\n");

    let mut file = std::fs::File::create(&src_file).unwrap();
    file.write_all(wrapped.as_bytes()).unwrap();

    let output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=bin")
        .arg("-o")
        .arg(tmp_dir.join("test_binary").to_str().unwrap())
        .arg(src_file.to_str().unwrap())
        .output()
        .expect("rustc not found");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "generated Rust for '{label}' does not compile:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn t095_csv_generated_code_compiles() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/csv_basic.pht"));
    assert_rustc_compiles(&code, "csv");
}

#[test]
fn t095_prompt_generated_code_compiles() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/prompt.pht"));
    assert_rustc_compiles(&code, "prompt");
}

#[test]
fn t095_config_generated_code_compiles() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/config.pht"));
    assert_rustc_compiles(&code, "config");
}

#[test]
fn t095_report_generated_code_compiles() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/report.pht"));
    assert_rustc_compiles(&code, "report");
}

// ═══════════════════════════════════════════════════════════════════════════
// T-096: Codegen runtime tests — compile, run, and verify rendered output
// ═══════════════════════════════════════════════════════════════════════════

/// Helper: compile and run generated Rust code, returning stdout.
fn compile_and_run(code: &str, main_code: &str, label: &str) -> String {
    use std::io::Write;

    let tmp_dir = std::env::temp_dir().join(format!("phenotyper-runtime-test-{label}"));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let src_file = tmp_dir.join("generated.rs");
    let binary = tmp_dir.join("test_binary");

    let wrapped = format!("{code}\nfn main() {{\n{main_code}\n}}\n");

    let mut file = std::fs::File::create(&src_file).unwrap();
    file.write_all(wrapped.as_bytes()).unwrap();

    let compile_output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("-o")
        .arg(binary.to_str().unwrap())
        .arg(src_file.to_str().unwrap())
        .output()
        .expect("rustc not found");

    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr);
    assert!(
        compile_output.status.success(),
        "compile failed for '{label}':\n{compile_stderr}"
    );

    let run_output = std::process::Command::new(&binary)
        .output()
        .expect("failed to run binary");

    assert!(run_output.status.success(), "runtime failed for '{label}'");

    let _ = std::fs::remove_dir_all(&tmp_dir);
    String::from_utf8(run_output.stdout).unwrap()
}

#[test]
fn t096_config_render_produces_correct_output() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/config.pht"));

    let main_code = r#"
        let entry = ConfigEntry {
            key: "host".to_string(),
            value: ConfigValue::String("localhost".to_string()),
            visibility: Visibility::Public,
        };
        let rendered = entry.render();
        print!("{}", rendered);
    "#;

    let output = compile_and_run(&code, main_code, "config-render");
    assert_eq!(output, "host = localhost [public]");
}

#[test]
fn t096_prompt_render_produces_correct_output() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/prompt.pht"));

    let main_code = r#"
        let section = PromptSection {
            heading: "Rules".to_string(),
            body: "Be concise.".to_string(),
        };
        let rendered = section.render();
        print!("{}", rendered);
    "#;

    let output = compile_and_run(&code, main_code, "prompt-render");
    assert_eq!(output, "## Rules\n\nBe concise.");
}

#[test]
fn t096_report_optional_fields_render() {
    let code = compile_ok(include_str!("../../../tests/fixtures/valid/report.pht"));

    // Test with subtitle present, tags empty, no footer
    let main_code = r#"
        let report = Report {
            title: "My Report".to_string(),
            subtitle: Some("Draft".to_string()),
            lines: ReportLines::from_vec(vec![
                ReportLine { content: "Line 1".to_string() },
                ReportLine { content: "Line 2".to_string() },
            ]),
            tags: Tags::new(),
            footer: None,
        };
        print!("{}", report.render());
    "#;

    let output = compile_and_run(&code, main_code, "report-optional");
    assert!(output.contains("# My Report\n"), "missing title");
    assert!(output.contains("## Draft\n"), "missing subtitle");
    assert!(output.contains("Line 1\nLine 2"), "missing lines");
    // Tags empty → @ifnotempty block should be absent
    assert!(
        !output.contains("Tags:"),
        "tags should be absent when empty"
    );
    // No footer
    assert!(!output.contains("---\n"), "footer should be absent");
}

// ═══════════════════════════════════════════════════════════════════════════
// T-268: Nested phenotypes end-to-end
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_nested_basic_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/nested_basic.pht");
    let code = compile_ok(source);

    // Both Parent and Child should be generated as flat types
    assert!(code.contains("pub struct Parent"), "missing Parent struct");
    assert!(code.contains("pub struct Child"), "missing Child struct");

    // Both should have builders
    assert!(
        code.contains("pub struct ParentBuilder"),
        "missing ParentBuilder"
    );
    assert!(
        code.contains("pub struct ChildBuilder"),
        "missing ChildBuilder"
    );

    // Both get Render impls
    assert!(
        code.contains("impl Render for Parent"),
        "missing Parent Render"
    );
    assert!(
        code.contains("impl Render for Child"),
        "missing Child Render"
    );
}

#[test]
fn e2e_javaclass_compiles() {
    let source = include_str!("../../../tests/fixtures/valid/javaclass.pht");
    let code = compile_ok(source);

    // Enum type
    assert!(
        code.contains("pub enum Visibility"),
        "missing Visibility enum"
    );

    // Parent type structures
    assert!(
        code.contains("pub struct JavaClass"),
        "missing JavaClass struct"
    );
    assert!(
        code.contains("pub struct JavaClasses"),
        "missing JavaClasses struct"
    );

    // Nested type structures (flattened)
    assert!(
        code.contains("pub struct Constructor"),
        "missing Constructor struct"
    );
    assert!(
        code.contains("pub struct Constructors"),
        "missing Constructors struct"
    );
}

#[test]
fn e2e_javaclass_has_render_with_parent() {
    let source = include_str!("../../../tests/fixtures/valid/javaclass.pht");
    let code = compile_ok(source);

    // Constructor references @(JavaClass/name) → generates render_with_parent
    assert!(
        code.contains("render_with_parent"),
        "missing render_with_parent method"
    );
    assert!(
        code.contains("parent: &JavaClass"),
        "missing parent parameter type"
    );
    // The parent field access
    assert!(code.contains("parent.name"), "missing parent.name access");
}

#[test]
fn e2e_nested_basic_no_parent_context() {
    let source = include_str!("../../../tests/fixtures/valid/nested_basic.pht");
    let code = compile_ok(source);

    // nested_basic has no parent-scoped refs, so no render_with_parent
    assert!(
        !code.contains("render_with_parent"),
        "unexpected render_with_parent — nested_basic has no parent refs"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PHT-0028: annotations are INERT under plain render — the golden
// ═══════════════════════════════════════════════════════════════════════════

/// The binding guarantee at its designed strength: the RENDER PATH of
/// an annotated template is byte-identical to its unannotated twin —
/// the `impl Render` blocks match exactly, and running both yields the
/// same bytes. (P2 adds `render_budgeted` beside render, so whole-file
/// code equality no longer holds — by design.)
#[test]
fn annotations_are_inert_under_plain_render() {
    let plain_src = r#"aivolution/test/annotations:

Evidence:
    hits: optional string,
    page: required string,
    stamp: required string,
    "e: ", @ifset(hits) { @(hits), " " }, @(page), @(stamp)
;
.
"#;
    let annotated_src = r#"aivolution/test/annotations:

Evidence:
    hits: optional string elastic(drop, priority 1),
    page: required string elastic(truncate, marker " [cut]") segment("page"),
    stamp: required string volatile,
    "e: ", @ifset(hits) { @(hits), " " }, @(page), @(stamp)
;
.
"#;
    let plain = compile_ok(plain_src);
    let annotated = compile_ok(annotated_src);
    assert_eq!(
        render_impl_blocks(&plain),
        render_impl_blocks(&annotated),
        "the plain render path must not change under annotations"
    );
    assert!(
        annotated.contains("pub fn render_budgeted"),
        "the annotated twin grows the budgeted renderer"
    );
    // And the rendered bytes agree, by running both.
    let main_code = r#"
    let e = Evidence { hits: Some("h".into()), page: "p".into(), stamp: "s".into() };
    print!("{}", e.render());
"#;
    let a = compile_and_run(&plain, main_code, "inert-plain");
    let b = compile_and_run(&annotated, main_code, "inert-annotated");
    assert_eq!(a, b, "render() output must be byte-identical");
}

/// Every `impl Render for … { … }` block, concatenated — the plain
/// render path, isolated from the budgeted additions.
fn render_impl_blocks(code: &str) -> String {
    let mut out = String::new();
    let mut lines = code.lines();
    while let Some(line) = lines.next() {
        if line.starts_with("impl Render for ") {
            out.push_str(line);
            out.push('\n');
            for l in lines.by_ref() {
                out.push_str(l);
                out.push('\n');
                if l == "}" {
                    break;
                }
            }
        }
    }
    out
}

#[test]
fn duplicate_annotations_are_refused() {
    let errs = compile_err(
        r#"aivolution/test/annotations:

Card:
    note: optional string elastic(drop, priority 1) elastic(drop, priority 2),
    @ifset(note) { @(note) }
;
.
"#,
    );
    assert!(
        errs.iter()
            .any(|e| e.summary.contains("declares `elastic` more than once")),
        "{errs:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PHT-0028 P2: the budgeted renderer, executed
// ═══════════════════════════════════════════════════════════════════════════

const BUDGET_SRC: &str = r##"aivolution/test/budget:

Hit plural Hits:
    line: required string,
    "\n- ", @(line)
;

Pack:
    title: required string segment("title"),
    guidance: optional string elastic(drop, priority 1),
    hits: required Hits elastic(drop, priority 2),
    page: required string elastic(truncate, marker "..."),
    clock: required string volatile,
    "# ", @(title),
    @ifset(guidance) { "\nG: ", @(guidance) },
    @join(hits, ""),
    "\nP: ", @(page),
    "\nT: ", @(clock)
;
.
"##;

fn budget_main() -> &'static str {
    r#"
    let pack = Pack {
        title: "T".into(),
        guidance: Some("be brief".into()),
        hits: Hits::from_vec(vec![
            Hit { line: "h1".into() },
            Hit { line: "h2".into() },
            Hit { line: "h3".into() },
        ]),
        page: "0123456789".into(),
        clock: "23:41".into(),
    };
    let len = |s: &str| s.len();
    let full = pack.render();

    // A roomy budget: zero degradation, text identical to render().
    let b = pack.render_budgeted(&Budget { limit: 10_000, measure: &len, floors: Floors::none() });
    assert_eq!(b.text, full);
    assert_eq!(b.cost, b.full_cost);
    assert!(b.dropped.is_empty() && b.truncated.is_empty());
    // Determinism: twice is identical.
    let b2 = pack.render_budgeted(&Budget { limit: 10_000, measure: &len, floors: Floors::none() });
    assert_eq!(b, b2);
    // The stable prefix ends where the clock's content begins.
    assert_eq!(b.stable_len, full.len() - "23:41".len());
    // The segment names the title's exact span.
    assert_eq!(b.segments.len(), 1);
    assert_eq!(&full[b.segments[0].1.clone()], "T");

    // Priority 1 drops first: a budget exactly without the guidance.
    let without_g = full.len() - "
G: be brief".len();
    let b = pack.render_budgeted(&Budget { limit: without_g, measure: &len, floors: Floors::none() });
    assert_eq!(b.dropped, vec!["guidance"]);
    assert!(b.truncated.is_empty());
    assert_eq!(b.cost, without_g);
    assert!(b.text.contains("- h3"));

    // Tighter: the hits tail drops next, later items first.
    let b = pack.render_budgeted(&Budget { limit: without_g - 3, measure: &len, floors: Floors::none() });
    assert!(b.dropped.contains(&"guidance") && b.dropped.contains(&"hits"));
    assert!(b.text.contains("- h1"));
    assert!(!b.text.contains("- h3"));
    assert!(b.cost <= without_g - 3);

    // Tighter still: the page truncates, marker appended.
    let b = pack.render_budgeted(&Budget { limit: 22, measure: &len, floors: Floors::none() });
    assert!(b.truncated.contains(&"page"), "{:?}", b);
    assert!(b.text.contains("..."));
    assert!(b.cost <= 22, "{} > 22", b.cost);

    // Floor honesty: an impossible budget reports cost > limit, equal
    // to the floor render, with a per-field floor holding five units
    // of page.
    let floors = Floors { default: 0, per_field: &[("page", 5)] };
    let b = pack.render_budgeted(&Budget { limit: 1, measure: &len, floors });
    assert!(b.cost > 1);
    assert_eq!(b.cost, b.floor_cost);
    let page_kept: String = b.text.chars().skip_while(|c| *c != 'P').collect();
    assert!(page_kept.len() >= 5, "the floor held: {page_kept}");

    // A non-additive measure (words): the guarantee still holds.
    let words = |s: &str| s.split_whitespace().count();
    let wfull = words(&full);
    let b = pack.render_budgeted(&Budget { limit: wfull - 2, measure: &words, floors: Floors::none() });
    assert!(b.cost <= wfull - 2, "{} > {}", b.cost, wfull - 2);

    print!("ok");
"#
}

#[test]
fn the_budgeted_renderer_holds_its_contracts() {
    let code = compile_ok(BUDGET_SRC);
    let out = compile_and_run(&code, budget_main(), "budgeted");
    assert_eq!(out, "ok");
}

// PHT-0029: a `segment` (or `volatile`) annotation on a NESTED-TYPE
// field, and on an optional string rendered inside its own `@ifset`
// (a guarded binding), hit an emitter path that wrote every value
// with `out.push_str(&…)` — right for an owned string, a type error
// for a nested type (no `Deref<str>`) and for a guarded `&String`
// (`&&String`). Field-found by aicogito SUM-0224: the records
// protocol is a nested phenotype and the daily bullet is a guarded
// optional string.
const NESTED_BUDGET_SRC: &str = r##"aivolution/test/nestedbudget:

Inner:
    body: required string,
    "[", @(body), "]"
;

Outer:
    inner: required Inner segment("inner"),
    extra: optional Inner segment("extra"),
    daily: optional string segment("daily"),
    "H: ",
    @(inner),
    @ifset(extra) { " E: ", @(extra) },
    @ifset(daily) { " D: ", @(daily) },
    "."
;
.
"##;

fn nested_budget_main() -> &'static str {
    r#"
    let full_pack = Outer {
        inner: Inner { body: "in".into() },
        extra: Some(Inner { body: "ex".into() }),
        daily: Some("01-Daily".into()),
    };
    let len = |s: &str| s.len();
    let full = full_pack.render();
    assert_eq!(full, "H: [in] E: [ex] D: 01-Daily.");

    // Roomy budget: byte-identical to render(), all three segments
    // naming their exact spans.
    let b = full_pack.render_budgeted(&Budget { limit: 10_000, measure: &len, floors: Floors::none() });
    assert_eq!(b.text, full);
    assert_eq!(b.segments.len(), 3);
    for (name, range) in &b.segments {
        let span = &full[range.clone()];
        match *name {
            "inner" => assert_eq!(span, "[in]"),
            "extra" => assert_eq!(span, "[ex]"),
            "daily" => assert_eq!(span, "01-Daily"),
            other => panic!("unexpected segment {other}"),
        }
    }

    // Unset optionals: no spans for them, text still identical.
    let bare = Outer {
        inner: Inner { body: "in".into() },
        extra: None,
        daily: None,
    };
    let b = bare.render_budgeted(&Budget { limit: 10_000, measure: &len, floors: Floors::none() });
    assert_eq!(b.text, bare.render());
    assert_eq!(b.segments.len(), 1);

    print!("ok");
"#
}

#[test]
fn segments_hold_on_nested_types_and_guarded_strings() {
    let code = compile_ok(NESTED_BUDGET_SRC);
    let out = compile_and_run(&code, nested_budget_main(), "nestedbudget");
    assert_eq!(out, "ok");
}
