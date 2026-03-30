// SPDX-License-Identifier: Apache-2.0
//! Build script for phenotyper-core — generates the Rustemo parser from the grammar.

use std::process::exit;

fn main() {
    let settings = rustemo_compiler::Settings::new()
        // Keep actions file in source tree (customizable, committed)
        // Parser goes to OUT_DIR (regenerated, not committed)
        .actions_in_source_tree()
        // Prefer shifts over empty reductions to resolve common LR conflicts
        .prefer_shifts_over_empty(true);

    if let Err(e) = settings.process_dir() {
        eprintln!("Rustemo grammar compilation failed:\n{e}");
        exit(1);
    }
}
