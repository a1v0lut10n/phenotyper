// SPDX-License-Identifier: Apache-2.0
//! CLI integration tests — test the `phenotyper` binary end-to-end (T-108).
//!
//! These tests invoke the compiled binary via `std::process::Command` and
//! verify exit codes, stdout, and stderr output.

use std::process::Command;

/// Get the path to the compiled `phenotyper` binary.
fn phenotyper_bin() -> std::path::PathBuf {
    // cargo test builds binaries into target/debug
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path.push("phenotyper");
    path
}

/// Get the path to a test fixture.
fn fixture(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../tests/fixtures/valid");
    path.push(name);
    path
}

/// Get the path to a doc example.
fn example(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../docs/examples");
    path.push(name);
    path
}

// ─── check subcommand ───────────────────────────────────────────────────────

#[test]
fn cli_check_valid_pht_exits_0() {
    let output = Command::new(phenotyper_bin())
        .args(["check", fixture("csv_basic.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("✓"), "expected success marker in stderr");
    assert!(stderr.contains("no errors"), "expected 'no errors'");
}

#[test]
fn cli_check_valid_md_exits_0() {
    let output = Command::new(phenotyper_bin())
        .args(["check", example("csv.md").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit code 0 for .md"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("✓"), "expected success marker");
}

#[test]
fn cli_check_invalid_source_exits_1() {
    // Create a temp file with invalid content
    let tmp = std::env::temp_dir().join("phenotyper-cli-test-invalid.pht");
    std::fs::write(&tmp, "this is not valid phenotyper").unwrap();

    let output = Command::new(phenotyper_bin())
        .args(["check", tmp.to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(1), "expected exit code 1");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"), "expected error in stderr");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn cli_check_nonexistent_file_exits_1() {
    let output = Command::new(phenotyper_bin())
        .args(["check", "nonexistent.pht"])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(1), "expected exit code 1");
}

#[test]
fn cli_check_unsupported_extension_exits_1() {
    let output = Command::new(phenotyper_bin())
        .args(["check", "file.txt"])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(1), "expected exit code 1");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported file extension"),
        "expected extension error"
    );
}

// ─── check --json ───────────────────────────────────────────────────────────

#[test]
fn cli_check_json_valid_no_output() {
    let output = Command::new(phenotyper_bin())
        .args([
            "check",
            "--json",
            fixture("csv_basic.pht").to_str().unwrap(),
        ])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0));
    // No diagnostics means no JSON output
    assert!(
        output.stdout.is_empty(),
        "expected no stdout for valid file with --json"
    );
}

#[test]
fn cli_check_json_invalid_outputs_json() {
    let tmp = std::env::temp_dir().join("phenotyper-cli-test-json.pht");
    std::fs::write(&tmp, "this is not valid").unwrap();

    let output = Command::new(phenotyper_bin())
        .args(["check", "--json", tmp.to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"severity\""),
        "expected JSON output on stdout"
    );
    assert!(stdout.contains("\"error\""), "expected error severity");

    let _ = std::fs::remove_file(&tmp);
}

// ─── build subcommand ───────────────────────────────────────────────────────

#[test]
fn cli_build_valid_creates_output() {
    let out_dir = std::env::temp_dir().join("phenotyper-cli-build-test");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(phenotyper_bin())
        .args([
            "build",
            fixture("csv_basic.pht").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("✓"), "expected success marker");
    assert!(stderr.contains("→"), "expected arrow pointing to output");

    // Verify output file exists
    let mod_rs = out_dir.join("aivolution/format/csv/mod.rs");
    assert!(mod_rs.exists(), "expected mod.rs to be created");

    let content = std::fs::read_to_string(&mod_rs).unwrap();
    assert!(
        content.contains("pub struct CsvFile"),
        "expected generated code"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn cli_build_md_creates_output() {
    let out_dir = std::env::temp_dir().join("phenotyper-cli-build-md-test");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(phenotyper_bin())
        .args([
            "build",
            example("csv.md").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit code 0 for .md build"
    );

    let mod_rs = out_dir.join("aivolution/format/csv/mod.rs");
    assert!(mod_rs.exists(), "expected mod.rs from .md build");

    let _ = std::fs::remove_dir_all(&out_dir);
}

// ─── dump-ast subcommand ────────────────────────────────────────────────────

#[test]
fn cli_dump_ast_outputs_debug() {
    let output = Command::new(phenotyper_bin())
        .args(["dump-ast", fixture("csv_basic.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File {"), "expected AST Debug output");
    assert!(
        stdout.contains("NamespaceScope"),
        "expected namespace in AST"
    );
}

// ─── dump-ir subcommand ─────────────────────────────────────────────────────

#[test]
fn cli_dump_ir_outputs_debug() {
    let output = Command::new(phenotyper_bin())
        .args(["dump-ir", fixture("csv_basic.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PhenotypeModule"),
        "expected IR Debug output"
    );
    assert!(
        stdout.contains("aivolution/format/csv"),
        "expected namespace in IR"
    );
}

// ─── usage errors (exit code 2 via clap) ────────────────────────────────────

#[test]
fn cli_no_subcommand_exits_2() {
    let output = Command::new(phenotyper_bin())
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 for usage error"
    );
}

#[test]
fn cli_unknown_subcommand_exits_2() {
    let output = Command::new(phenotyper_bin())
        .args(["unknown"])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 for unknown subcommand"
    );
}

#[test]
fn cli_check_missing_file_arg_exits_2() {
    let output = Command::new(phenotyper_bin())
        .args(["check"])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2 for missing arg"
    );
}

// ─── nested phenotype files (M13) ──────────────────────────────────────────

#[test]
fn cli_check_nested_basic_exits_0() {
    let output = Command::new(phenotyper_bin())
        .args(["check", fixture("nested_basic.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("✓"), "expected success marker");
    assert!(stderr.contains("no errors"), "expected 'no errors'");
}

#[test]
fn cli_check_javaclass_exits_0() {
    let output = Command::new(phenotyper_bin())
        .args(["check", fixture("javaclass.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("✓"), "expected success marker");
}

#[test]
fn cli_build_nested_basic_creates_output() {
    let out_dir = std::env::temp_dir().join("phenotyper-cli-nested-test");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(phenotyper_bin())
        .args([
            "build",
            fixture("nested_basic.pht").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");

    // Verify output file exists
    let mod_rs = out_dir.join("test/nested/mod.rs");
    assert!(mod_rs.exists(), "expected mod.rs to be created");

    let content = std::fs::read_to_string(&mod_rs).unwrap();
    assert!(
        content.contains("pub struct Parent"),
        "expected Parent struct"
    );
    assert!(
        content.contains("pub struct Child"),
        "expected Child struct"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn cli_build_javaclass_creates_output() {
    let out_dir = std::env::temp_dir().join("phenotyper-cli-javaclass-test");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(phenotyper_bin())
        .args([
            "build",
            fixture("javaclass.pht").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0), "expected exit code 0");

    let mod_rs = out_dir.join("java/types/mod.rs");
    assert!(mod_rs.exists(), "expected mod.rs to be created");

    let content = std::fs::read_to_string(&mod_rs).unwrap();
    assert!(
        content.contains("pub struct JavaClass"),
        "expected JavaClass struct"
    );
    assert!(
        content.contains("pub struct Constructor"),
        "expected Constructor struct"
    );
    assert!(
        content.contains("render_with_parent"),
        "expected render_with_parent"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn cli_dump_ast_nested_outputs_nested_type() {
    let output = Command::new(phenotyper_bin())
        .args(["dump-ast", fixture("nested_basic.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("NestedType"),
        "expected NestedType in AST debug"
    );
}

#[test]
fn cli_dump_ir_javaclass_shows_parent_ref() {
    let output = Command::new(phenotyper_bin())
        .args(["dump-ir", fixture("javaclass.pht").to_str().unwrap()])
        .output()
        .expect("failed to run phenotyper");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ParentFieldRef"),
        "expected ParentFieldRef in IR"
    );
    assert!(
        stdout.contains("parent_context: Some"),
        "expected parent_context in IR"
    );
}
