// SPDX-License-Identifier: Apache-2.0
//! Lexer module — tokenization and source extraction.

/// Source span tracking original file positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Source file path (or identifier).
    pub file: String,
    /// Start line (1-indexed).
    pub start_line: usize,
    /// Start column (1-indexed).
    pub start_col: usize,
    /// End line (1-indexed).
    pub end_line: usize,
    /// End column (1-indexed).
    pub end_col: usize,
}
