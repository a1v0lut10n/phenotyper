// SPDX-License-Identifier: Apache-2.0
//! `uses` end to end (PHT-0027, REQ-LANG-003): compilation sets over search
//! roots, cross-namespace codegen, the mountable module tree — proven by
//! compiling and running the generated code with `rustc`, like the other
//! e2e tests.

use std::path::{Path, PathBuf};

const BADGE_DEP: &str = r#"test/core/badge:
Badge plural Badges:
    label: required string,
    @(label)
;
.
"#;

const CARD_ENTRY: &str = r#"test/ai/card:
uses test/core/badge;
Card:
    title: required string,
    badge: required Badge,
    @(title), " [", @(badge), "]"
;
.
"#;

/// A fresh scratch directory per test.
fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "phenotyper-uses-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write `source` at `<root>/<ns>.pht`.
fn write_ns(root: &Path, ns: &str, source: &str) -> PathBuf {
    let path = root.join(format!("{ns}.pht"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    path
}

#[test]
fn compile_set_resolves_uses_and_writes_the_module_tree() {
    let dir = scratch("happy");
    let root = dir.join("src");
    let out = dir.join("out");
    write_ns(&root, "test/core/badge", BADGE_DEP);
    let entry = write_ns(&root, "test/ai/card", CARD_ENTRY);

    let outputs = phenotyper::compile_with_roots(&entry, &[&root], &out)
        .unwrap_or_else(|e| panic!("compile failed: {e:#?}"));

    // Dependency order: the dep first, the entry last.
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].namespace, "test/core/badge");
    assert_eq!(outputs[1].namespace, "test/ai/card");

    // Cross-namespace codegen: relative path and UFCS render.
    let card_code = &outputs[1].code;
    assert!(
        card_code.contains("super::super::core::badge::Badge"),
        "imported type path missing:\n{card_code}"
    );
    assert!(
        card_code.contains("Render::render_into(&self.badge, out)"),
        "UFCS render call missing:\n{card_code}"
    );

    // The mountable module tree.
    let read = |p: &Path| std::fs::read_to_string(p).unwrap();
    assert!(read(&out.join("mod.rs")).contains("pub mod test;"));
    let test_mod = read(&out.join("test/mod.rs"));
    assert!(test_mod.contains("pub mod ai;") && test_mod.contains("pub mod core;"));
    assert!(read(&out.join("test/ai/mod.rs")).contains("pub mod card;"));
    assert!(read(&out.join("test/core/mod.rs")).contains("pub mod badge;"));
    assert!(out.join("test/ai/card/mod.rs").is_file());
    assert!(out.join("test/core/badge/mod.rs").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

/// The generated tree compiles with rustc and renders across the namespace
/// boundary — the whole point of `uses`.
#[test]
fn generated_cross_namespace_code_compiles_and_renders() {
    let dir = scratch("rustc");
    let root = dir.join("src");
    let out = dir.join("out");
    write_ns(&root, "test/core/badge", BADGE_DEP);
    let entry = write_ns(&root, "test/ai/card", CARD_ENTRY);
    phenotyper::compile_with_roots(&entry, &[&root], &out)
        .unwrap_or_else(|e| panic!("compile failed: {e:#?}"));

    // A main.rs beside the tree mounts it the way a consumer would.
    let main_rs = out.join("main.rs");
    std::fs::write(
        &main_rs,
        r#"pub mod test;

fn main() {
    use test::ai::card::Render;
    let card = test::ai::card::Card {
        title: "Genite".to_string(),
        badge: test::core::badge::Badge { label: "gold".to_string() },
    };
    print!("{}", card.render());
}
"#,
    )
    .unwrap();

    let binary = dir.join("uses_e2e");
    let compile = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("-o")
        .arg(&binary)
        .arg(&main_rs)
        .output()
        .expect("rustc not found");
    assert!(
        compile.status.success(),
        "rustc failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = std::process::Command::new(&binary).output().unwrap();
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout), "Genite [gold]");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_uses_cycle_is_a_named_error() {
    let dir = scratch("cycle");
    let root = dir.join("src");
    write_ns(
        &root,
        "test/a",
        "test/a:\nuses test/b;\nA:\n    x: required string,\n    @(x)\n;\n.\n",
    );
    let entry = write_ns(
        &root,
        "test/b",
        "test/b:\nuses test/a;\nB:\n    y: required string,\n    @(y)\n;\n.\n",
    );

    let err = phenotyper::compile_with_roots(&entry, &[&root], dir.join("out"))
        .expect_err("cycle must fail");
    assert!(
        err.iter().any(|d| d.summary.contains("cycle in `uses`")),
        "expected cycle error: {err:#?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_dependency_names_the_roots() {
    let dir = scratch("missing");
    let root = dir.join("src");
    let entry = write_ns(&root, "test/ai/card", CARD_ENTRY);

    let err = phenotyper::compile_with_roots(&entry, &[&root], dir.join("out"))
        .expect_err("missing dep must fail");
    assert!(
        err.iter()
            .any(|d| d.summary.contains("cannot resolve `uses test/core/badge;`")),
        "expected missing-dep error: {err:#?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_namespace_file_mismatch_is_an_error() {
    let dir = scratch("mismatch");
    let root = dir.join("src");
    // The file sits where test/core/badge should be but declares another ns.
    write_ns(
        &root,
        "test/core/badge",
        "test/core/sticker:\nSticker:\n    s: required string,\n    @(s)\n;\n.\n",
    );
    let entry = write_ns(&root, "test/ai/card", CARD_ENTRY);

    let err = phenotyper::compile_with_roots(&entry, &[&root], dir.join("out"))
        .expect_err("mismatch must fail");
    assert!(
        err.iter()
            .any(|d| d.summary.contains("expected namespace `test/core/badge`")),
        "expected mismatch error: {err:#?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `compile_source` (single-file) stays honest about `uses`.
#[test]
fn single_file_compile_with_uses_points_at_roots() {
    let err = phenotyper::compile_source(CARD_ENTRY, "card.pht", None)
        .expect_err("must fail without roots");
    assert!(
        err.iter()
            .any(|d| d.summary.contains("cannot resolve `uses test/core/badge;`")),
        "expected unresolved-uses error: {err:#?}"
    );
}
