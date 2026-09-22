// SPDX-License-Identifier: Apache-2.0
//! Phenotyper Core — compiler library for the Phenotyper structural artifact definition language.
//!
//! This crate implements the full compilation pipeline:
//! - **Lexer**: Tokenization of `.pht` and `.md` sources
//! - **Parser**: Rustemo-based parser producing a span-annotated AST
//! - **AST**: Surface syntax tree data types
//! - **Symbol table**: Name collection and resolution
//! - **IR**: Normalized intermediate representation
//! - **Semantic validation**: Type, render, and generation validation
//! - **Codegen**: Rust source code generation
//!
//! # Build script integration
//!
//! For `build.rs` integration (like `prost-build` or `rustemo`), use the
//! high-level [`compile()`] function:
//!
//! ```ignore
//! // In your build.rs:
//! fn main() {
//!     phenotyper::compile("src/phenotypes/csv.pht", "src/generated")
//!         .expect("phenotyper compilation failed");
//! }
//! ```

pub mod ast;
pub mod codegen;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod symbol;

use std::path::Path;

use diagnostic::Diagnostic;

// ─── High-level compile API ─────────────────────────────────────────────────

/// Result type for compilation.
pub type CompileResult = Result<CompileOutput, Vec<Diagnostic>>;

/// Successful compilation output.
#[derive(Debug)]
pub struct CompileOutput {
    /// The generated Rust source code.
    pub code: String,
    /// The namespace path (e.g., `"aivolution/format/csv"`).
    pub namespace: String,
    /// Any warnings generated during compilation.
    pub warnings: Vec<Diagnostic>,
}

/// Compile a Phenotyper source file and generate Rust code.
///
/// This is the primary entry point for `build.rs` integration. It runs the
/// full compilation pipeline (parse → symbols → IR → validate → codegen)
/// and returns the generated Rust code.
///
/// # Arguments
///
/// * `source_path` — Path to the `.pht` or `.md` source file.
/// * `out_dir` — Directory where generated Rust code will be written.
///
/// # Cargo integration
///
/// This function automatically prints `cargo:rerun-if-changed` directives
/// to stdout so Cargo will re-run the build script when the source changes.
///
/// # Errors
///
/// Returns a `Vec<Diagnostic>` if any phase produces errors.
///
/// # Example
///
/// ```ignore
/// // build.rs
/// fn main() {
///     phenotyper::compile("src/phenotypes/csv.pht", "src/generated")
///         .expect("phenotyper compilation failed");
/// }
/// ```
pub fn compile(source_path: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> CompileResult {
    let source_path = source_path.as_ref();
    let out_dir = out_dir.as_ref();
    let file_str = source_path.display().to_string();

    // Emit cargo:rerun-if-changed for build.rs integration
    println!("cargo:rerun-if-changed={file_str}");

    // Read source
    let source = std::fs::read_to_string(source_path)
        .map_err(|e| vec![Diagnostic::error(&file_str, format!("cannot read: {e}"))])?;

    compile_source(&source, &file_str, Some(out_dir))
}

/// Compile Phenotyper source code from a string (no file I/O for the source).
///
/// Useful for testing or embedding. If `out_dir` is `Some`, the generated
/// code is written to the filesystem; otherwise it is only returned.
///
/// A source with `uses` declarations cannot be compiled standalone — each
/// one becomes an error pointing at [`compile_with_roots`].
pub fn compile_source(source: &str, file_name: &str, out_dir: Option<&Path>) -> CompileResult {
    compile_one(source, file_name, out_dir, &[]).map(|(output, _exports)| output)
}

/// The single-file pipeline, with the exports of the `uses` namespaces in
/// scope. Returns the output plus this namespace's own exports, so a
/// compilation set can offer them to *its* importers.
fn compile_one(
    source: &str,
    file_name: &str,
    out_dir: Option<&Path>,
    available: &[symbol::NamespaceExports],
) -> Result<(CompileOutput, symbol::NamespaceExports), Vec<Diagnostic>> {
    let mut warnings = Vec::new();

    // Parse — auto-detect .md vs .pht by file extension
    let is_md = file_name.ends_with(".md");
    let ast = if is_md {
        parser::parse_md(source, file_name)?
    } else {
        parser::parse_pht(source, file_name)?
    };

    // Symbol resolution (imports first, then collection and resolution)
    let (table, sym_diags) = symbol::build_with_imports(&ast, file_name, available);
    let (sym_errors, sym_warnings): (Vec<_>, Vec<_>) =
        sym_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(sym_warnings);
    if !sym_errors.is_empty() {
        return Err(sym_errors);
    }

    // IR lowering
    let (module, ir_diags) = ir::lower(&ast, &table, file_name);
    let (ir_errors, ir_warnings): (Vec<_>, Vec<_>) =
        ir_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(ir_warnings);
    if !ir_errors.is_empty() {
        return Err(ir_errors);
    }

    // Semantic validation
    let sem_diags = semantic::validate(&module, file_name);
    let (sem_errors, sem_warnings): (Vec<_>, Vec<_>) =
        sem_diags.into_iter().partition(|d| d.is_error());
    warnings.extend(sem_warnings);
    if !sem_errors.is_empty() {
        return Err(sem_errors);
    }

    // Code generation
    let code = codegen::generate(&module);

    // Write output if directory specified
    if let Some(out_dir) = out_dir {
        let ns_path = module.namespace.replace('/', std::path::MAIN_SEPARATOR_STR);
        let dir = out_dir.join(&ns_path);
        let file = dir.join("mod.rs");

        std::fs::create_dir_all(&dir).map_err(|e| {
            vec![Diagnostic::error(
                file_name,
                format!("cannot create directory '{}': {e}", dir.display()),
            )]
        })?;

        std::fs::write(&file, &code).map_err(|e| {
            vec![Diagnostic::error(
                file_name,
                format!("cannot write '{}': {e}", file.display()),
            )]
        })?;
    }

    let exports = table.exports();
    Ok((
        CompileOutput {
            code,
            namespace: module.namespace.clone(),
            warnings,
        },
        exports,
    ))
}

// ─── Compilation sets (`uses`, REQ-LANG-003) ────────────────────────────────

/// Compile a source file **and every namespace it transitively `uses`**,
/// resolving each `uses a/b/c;` against `roots`: the first root containing
/// `a/b/c.pht` or `a/b/c.md` wins.
///
/// Each namespace is compiled once, in dependency order (the entry file's
/// output comes last), and written to `out_dir/<namespace>/mod.rs`. The
/// intermediate directories get `mod.rs` files declaring their submodules,
/// so the whole tree mounts into a consumer crate with a single
/// `#[path] mod` (or `include!`) of `out_dir/mod.rs` — which is what makes
/// the generated cross-namespace `super::…` paths hold.
///
/// `uses` cycles are a hard error naming the cycle.
pub fn compile_with_roots(
    source_path: impl AsRef<Path>,
    roots: &[impl AsRef<Path>],
    out_dir: impl AsRef<Path>,
) -> Result<Vec<CompileOutput>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let out_dir = out_dir.as_ref();
    let roots: Vec<&Path> = roots.iter().map(AsRef::as_ref).collect();

    let mut set = CompileSet {
        roots,
        out_dir,
        compiled: Vec::new(),
        exports: Vec::new(),
        in_progress: Vec::new(),
    };

    set.compile_file(source_path, None)?;

    let namespaces: Vec<String> = set.compiled.iter().map(|o| o.namespace.clone()).collect();
    write_module_tree(out_dir, &namespaces)?;

    Ok(set.compiled)
}

/// The state of one `compile_with_roots` run.
struct CompileSet<'a> {
    roots: Vec<&'a Path>,
    out_dir: &'a Path,
    /// Outputs in dependency (post) order.
    compiled: Vec<CompileOutput>,
    /// Exports of every compiled namespace.
    exports: Vec<symbol::NamespaceExports>,
    /// The `uses` chain currently being resolved, for cycle reporting.
    in_progress: Vec<String>,
}

impl CompileSet<'_> {
    /// Compile one file (recursing into its `uses` first). `expected_ns` is
    /// set when the file was located *for* a `uses` declaration.
    fn compile_file(
        &mut self,
        path: &Path,
        expected_ns: Option<&str>,
    ) -> Result<(), Vec<Diagnostic>> {
        let file_str = path.display().to_string();
        println!("cargo:rerun-if-changed={file_str}");

        let source = std::fs::read_to_string(path)
            .map_err(|e| vec![Diagnostic::error(&file_str, format!("cannot read: {e}"))])?;

        // Parse once to learn the namespace and its `uses`.
        let is_md = file_str.ends_with(".md");
        let ast = if is_md {
            parser::parse_md(&source, &file_str)?
        } else {
            parser::parse_pht(&source, &file_str)?
        };
        let ns = ast.ns.path.join("/");

        if let Some(expected) = expected_ns {
            if ns != expected {
                return Err(vec![Diagnostic::error(
                    &file_str,
                    format!("expected namespace `{expected}` but the file declares `{ns}`"),
                )]);
            }
        }
        if self.exports.iter().any(|e| e.namespace == ns) {
            return Ok(()); // already compiled through another `uses` path
        }
        if let Some(pos) = self.in_progress.iter().position(|n| n == &ns) {
            let mut cycle = self.in_progress[pos..].to_vec();
            cycle.push(ns.clone());
            return Err(vec![Diagnostic::error(
                &file_str,
                format!("cycle in `uses`: {}", cycle.join(" -> ")),
            )]);
        }

        // Dependencies first.
        self.in_progress.push(ns.clone());
        let dep_namespaces: Vec<String> = ast
            .ns
            .uses
            .iter()
            .flatten()
            .map(|u| u.path.join("/"))
            .filter(|dep| *dep != ns)
            .collect();
        for dep in &dep_namespaces {
            if self.exports.iter().any(|e| e.namespace == *dep) {
                continue;
            }
            let dep_path = self.locate(dep).ok_or_else(|| {
                vec![Diagnostic::error(
                    &file_str,
                    format!(
                        "cannot resolve `uses {dep};` — no `{dep}.pht` or `{dep}.md` under {}",
                        self.roots
                            .iter()
                            .map(|r| r.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )]
            })?;
            self.compile_file(&dep_path, Some(dep))?;
        }
        self.in_progress.pop();

        // Then this namespace, with its dependencies' exports in scope.
        let (output, exports) = compile_one(&source, &file_str, Some(self.out_dir), &self.exports)?;
        self.compiled.push(output);
        self.exports.push(exports);
        Ok(())
    }

    /// Find the source file for a namespace under the search roots.
    fn locate(&self, ns: &str) -> Option<std::path::PathBuf> {
        let rel = ns.replace('/', std::path::MAIN_SEPARATOR_STR);
        for root in &self.roots {
            for ext in ["pht", "md"] {
                let candidate = root.join(&rel).with_extension(ext);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }
}

/// Write the `mod.rs` chain that turns the per-namespace outputs into one
/// mountable module tree: every intermediate directory declares its
/// children, and a namespace that is itself an ancestor of another gets the
/// `pub mod` lines appended to its generated code.
fn write_module_tree(out_dir: &Path, namespaces: &[String]) -> Result<(), Vec<Diagnostic>> {
    use std::collections::{BTreeMap, BTreeSet};

    // prefix ("" = the out_dir root) → child module names
    let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for ns in namespaces {
        let segments: Vec<&str> = ns.split('/').collect();
        let mut prefix = String::new();
        for segment in &segments {
            children
                .entry(prefix.clone())
                .or_default()
                .insert((*segment).to_string());
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(segment);
        }
    }

    let io_err = |path: &Path, e: std::io::Error| {
        vec![Diagnostic::error(
            path.display().to_string(),
            format!("cannot write module tree: {e}"),
        )]
    };

    for (prefix, kids) in &children {
        let dir = if prefix.is_empty() {
            out_dir.to_path_buf()
        } else {
            out_dir.join(prefix.replace('/', std::path::MAIN_SEPARATOR_STR))
        };
        let file = dir.join("mod.rs");
        let decls: String = kids.iter().map(|k| format!("pub mod {k};\n")).collect();

        if namespaces.iter().any(|ns| ns == prefix) {
            // This directory is a compiled namespace: append the submodule
            // declarations to its generated code (idempotent per run — the
            // namespace file was just rewritten by compile_one).
            let mut code = std::fs::read_to_string(&file).map_err(|e| io_err(&file, e))?;
            code.push('\n');
            code.push_str(&decls);
            std::fs::write(&file, code).map_err(|e| io_err(&file, e))?;
        } else {
            std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
            let header = "// This file is @generated by phenotyper. Do not edit.\n\n";
            std::fs::write(&file, format!("{header}{decls}")).map_err(|e| io_err(&file, e))?;
        }
    }
    Ok(())
}
