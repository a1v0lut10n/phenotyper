// SPDX-License-Identifier: Apache-2.0
//! Markdown source extraction — extract `pht` fenced code blocks and build a source map.
//!
//! When the compiler processes a `.md` file, it locates all fenced code blocks
//! tagged with `pht` and extracts their contents. A [`SourceMap`] is built so
//! that diagnostics can reference the original `.md` line numbers.

/// A single extracted block of Phenotyper source from a Markdown file.
#[derive(Debug, Clone)]
pub struct SourceBlock {
    /// The extracted source text (without the fence lines themselves).
    pub text: String,
    /// The 1-indexed line in the original `.md` file where the content starts
    /// (the line immediately after the opening fence).
    pub md_start_line: usize,
}

/// Source map for Markdown-embedded Phenotyper sources.
///
/// Maps line numbers in the concatenated extracted text back to the original
/// `.md` line numbers.
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// The concatenated source text of all extracted blocks.
    pub source: String,
    /// For each line in `source` (0-indexed), the corresponding 1-indexed
    /// line number in the original `.md` file.
    pub line_map: Vec<usize>,
}

impl SourceMap {
    /// Translate a 1-indexed line number in the extracted source to the
    /// corresponding 1-indexed line number in the original `.md` file.
    pub fn original_line(&self, extracted_line: usize) -> usize {
        let idx = extracted_line.saturating_sub(1);
        if idx < self.line_map.len() {
            self.line_map[idx]
        } else {
            // Beyond the map — return the last known mapping.
            self.line_map.last().copied().unwrap_or(extracted_line)
        }
    }
}

/// Extract all `pht` fenced code blocks from a Markdown string.
///
/// Returns the list of individual blocks and a [`SourceMap`] that maps
/// concatenated-source lines back to original `.md` lines.
pub fn extract_pht_blocks(markdown: &str) -> (Vec<SourceBlock>, SourceMap) {
    let mut blocks = Vec::new();
    let mut source = String::new();
    let mut line_map: Vec<usize> = Vec::new();

    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;
    let mut in_block = false;
    let mut block_text = String::new();
    let mut block_start_line: usize = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if !in_block {
            // Look for opening fence: ```pht or ~~~pht (with optional whitespace)
            if is_pht_fence_open(trimmed) {
                in_block = true;
                block_text.clear();
                block_start_line = i + 2; // next line (1-indexed)
            }
        } else {
            // Look for closing fence
            if is_fence_close(trimmed) {
                // End of block
                blocks.push(SourceBlock {
                    text: block_text.clone(),
                    md_start_line: block_start_line,
                });

                // Append to concatenated source (with newline between blocks)
                if !source.is_empty() {
                    source.push('\n');
                    // The newline between blocks maps to the closing fence line
                    line_map.push(i + 1);
                }
                source.push_str(&block_text);

                in_block = false;
            } else {
                // Content line inside a pht block
                if !block_text.is_empty() {
                    block_text.push('\n');
                }
                block_text.push_str(line);
                line_map.push(i + 1); // 1-indexed .md line
            }
        }

        i += 1;
    }

    // If we ended inside an unclosed block, still include what we have
    if in_block && !block_text.is_empty() {
        blocks.push(SourceBlock {
            text: block_text.clone(),
            md_start_line: block_start_line,
        });
        if !source.is_empty() {
            source.push('\n');
            line_map.push(block_start_line);
        }
        source.push_str(&block_text);
    }

    let map = SourceMap { source, line_map };
    (blocks, map)
}

/// Check if a trimmed line is a `pht` opening fence.
fn is_pht_fence_open(trimmed: &str) -> bool {
    // Accept ```pht, ``` pht, ~~~pht, ~~~ pht (case-insensitive tag)
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim().eq_ignore_ascii_case("pht");
    }
    if let Some(rest) = trimmed.strip_prefix("~~~") {
        return rest.trim().eq_ignore_ascii_case("pht");
    }
    false
}

/// Check if a trimmed line is a closing fence.
fn is_fence_close(trimmed: &str) -> bool {
    trimmed == "```" || trimmed == "~~~"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_block() {
        let md = r#"# Example

```pht
namespace aivolution/format/csv;
```

Some text.
"#;
        let (blocks, map) = extract_pht_blocks(md);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].text,
            "namespace aivolution/format/csv;"
        );
        assert_eq!(blocks[0].md_start_line, 4); // line 4 in the .md
        assert_eq!(map.source, "namespace aivolution/format/csv;");
        assert_eq!(map.original_line(1), 4);
    }

    #[test]
    fn extract_multiple_blocks() {
        let md = r#"# Part 1

```pht
namespace foo/bar;
```

# Part 2

```pht
CSVFile:
    header: required string,
    @(header)
;
```
"#;
        let (blocks, map) = extract_pht_blocks(md);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, "namespace foo/bar;");
        assert!(blocks[1].text.contains("CSVFile:"));

        // Line 1 in extracted source = line 4 in .md
        assert_eq!(map.original_line(1), 4);
    }

    #[test]
    fn extract_no_blocks() {
        let md = "# No pht here\n\nJust text.\n";
        let (blocks, map) = extract_pht_blocks(md);
        assert_eq!(blocks.len(), 0);
        assert!(map.source.is_empty());
    }

    #[test]
    fn extract_tilde_fence() {
        let md = "~~~pht\nfoo: required string;\n~~~\n";
        let (blocks, _) = extract_pht_blocks(md);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "foo: required string;");
    }
}
