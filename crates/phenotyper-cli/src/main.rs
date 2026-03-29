// SPDX-License-Identifier: Apache-2.0
//! Phenotyper CLI — compiler binary for the Phenotyper language.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "phenotyper", about = "Phenotyper v1 compiler", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a phenotyper source file for errors without generating code.
    Check {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output diagnostics as JSON
        #[arg(long)]
        json: bool,
    },

    /// Compile a phenotyper source file and generate Rust code.
    Build {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output directory for generated Rust code
        #[arg(long, short, value_name = "DIR")]
        out: PathBuf,

        /// Output diagnostics as JSON
        #[arg(long)]
        json: bool,
    },

    /// Dump the AST of a source file (debug).
    DumpAst {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Dump the IR of a source file (debug).
    DumpIr {
        /// Source file (.pht or .md)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { file, json: _ } => {
            eprintln!("phenotyper check: {:?} (not yet implemented)", file);
            ExitCode::from(1)
        }
        Commands::Build { file, out, json: _ } => {
            eprintln!(
                "phenotyper build: {:?} -> {:?} (not yet implemented)",
                file, out
            );
            ExitCode::from(1)
        }
        Commands::DumpAst { file } => {
            eprintln!("phenotyper dump-ast: {:?} (not yet implemented)", file);
            ExitCode::from(1)
        }
        Commands::DumpIr { file } => {
            eprintln!("phenotyper dump-ir: {:?} (not yet implemented)", file);
            ExitCode::from(1)
        }
    }
}
