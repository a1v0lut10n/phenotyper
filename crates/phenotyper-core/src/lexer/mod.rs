// SPDX-License-Identifier: Apache-2.0
//! Lexer module — tokenization and source extraction for Phenotyper v1.
//!
//! The lexer converts raw source text (`.pht` or extracted from `.md`) into a
//! stream of [`SpannedToken`]s. It handles:
//! - All v1 keywords and primitive type names (lexically reserved)
//! - Identifiers, string literals with escape sequences
//! - Structural punctuation and operators
//! - Line and block comments (discarded)
//! - Whitespace (discarded)

pub mod source_map;
mod token;

pub use source_map::{SourceBlock, SourceMap, extract_pht_blocks};
pub use token::{Keyword, Span, SpannedToken, Token};

use crate::diagnostic::{Diagnostic, Severity};

/// Lex an input string into a sequence of spanned tokens.
///
/// Comments and whitespace are consumed but not emitted.
/// Returns `(tokens, diagnostics)`. If diagnostics contain errors,
/// the token stream may be incomplete.
pub fn lex(source: &str, file: &str) -> (Vec<SpannedToken>, Vec<Diagnostic>) {
    let mut lexer = LexerState::new(source, file);
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    loop {
        lexer.skip_whitespace();
        if lexer.is_eof() {
            tokens.push(lexer.make_token(Token::Eof, 0));
            break;
        }

        match lexer.next_token() {
            Ok(Some(tok)) => tokens.push(tok),
            Ok(None) => { /* comment or whitespace, already consumed */ }
            Err(diag) => {
                diagnostics.push(diag);
                // Skip the offending character and continue.
                lexer.advance();
            }
        }
    }

    (tokens, diagnostics)
}

// ---------------------------------------------------------------------------
// Internal lexer state
// ---------------------------------------------------------------------------

struct LexerState<'a> {
    source: &'a str,
    file: String,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> LexerState<'a> {
    fn new(source: &'a str, file: &str) -> Self {
        Self {
            source,
            file: file.to_string(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.pos..].chars();
        chars.next();
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn make_token(&self, kind: Token, len: usize) -> SpannedToken {
        // `len` is the number of source characters consumed for this token.
        // The span end column points to the last character of the token.
        let end_col = if len > 0 { self.col - 1 } else { self.col };
        SpannedToken {
            token: kind,
            span: Span {
                file: self.file.clone(),
                start_line: self.line,
                start_col: if len > 0 {
                    self.col.saturating_sub(len)
                } else {
                    self.col
                },
                end_line: self.line,
                end_col,
            },
        }
    }

    fn make_span_from(&self, start_line: usize, start_col: usize) -> Span {
        Span {
            file: self.file.clone(),
            start_line,
            start_col,
            end_line: self.line,
            end_col: if self.col > 1 { self.col - 1 } else { 1 },
        }
    }

    fn error_at(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            summary: message.into(),
            file: self.file.clone(),
            line: self.line,
            col: self.col,
            explanation: None,
            suggestion: None,
        }
    }

    /// Try to consume the next token.
    /// Returns:
    /// - `Ok(Some(token))` for a real token
    /// - `Ok(None)` if a comment was consumed (caller should continue)
    /// - `Err(diagnostic)` on error
    fn next_token(&mut self) -> Result<Option<SpannedToken>, Diagnostic> {
        let start_line = self.line;
        let start_col = self.col;
        let ch = self.peek().unwrap();

        match ch {
            // --- String literals ---
            '"' => self.lex_string_literal(start_line, start_col).map(Some),

            // --- Two-character lookahead: comments and slash ---
            '/' => {
                match self.peek_next() {
                    Some('/') => {
                        self.lex_line_comment();
                        Ok(None)
                    }
                    Some('*') => {
                        self.lex_block_comment()?;
                        Ok(None)
                    }
                    _ => {
                        // Slash for namespace paths
                        self.advance();
                        Ok(Some(SpannedToken {
                            token: Token::Slash,
                            span: self.make_span_from(start_line, start_col),
                        }))
                    }
                }
            }

            // --- Single-character tokens ---
            ':' => self.single_char_token(Token::Colon, start_line, start_col),
            ';' => self.single_char_token(Token::Semicolon, start_line, start_col),
            ',' => self.single_char_token(Token::Comma, start_line, start_col),
            '{' => self.single_char_token(Token::LeftBrace, start_line, start_col),
            '}' => self.single_char_token(Token::RightBrace, start_line, start_col),
            '[' => self.single_char_token(Token::LeftBracket, start_line, start_col),
            ']' => self.single_char_token(Token::RightBracket, start_line, start_col),
            '(' => self.single_char_token(Token::LeftParen, start_line, start_col),
            ')' => self.single_char_token(Token::RightParen, start_line, start_col),
            '@' => self.single_char_token(Token::At, start_line, start_col),
            '+' => self.single_char_token(Token::Plus, start_line, start_col),
            '*' => self.single_char_token(Token::Star, start_line, start_col),

            // --- Identifiers and keywords ---
            c if is_ident_start(c) => Ok(Some(self.lex_identifier(start_line, start_col))),

            // --- Unknown character ---
            _ => Err(self.error_at(format!("unexpected character: '{ch}'"))),
        }
    }

    fn single_char_token(
        &mut self,
        kind: Token,
        start_line: usize,
        start_col: usize,
    ) -> Result<Option<SpannedToken>, Diagnostic> {
        self.advance();
        Ok(Some(SpannedToken {
            token: kind,
            span: self.make_span_from(start_line, start_col),
        }))
    }

    fn lex_identifier(&mut self, start_line: usize, start_col: usize) -> SpannedToken {
        let start_pos = self.pos;
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start_pos..self.pos];
        let token = match classify_keyword(text) {
            Some(kw) => Token::Keyword(kw),
            None => Token::Identifier(text.to_string()),
        };
        SpannedToken {
            token,
            span: self.make_span_from(start_line, start_col),
        }
    }

    fn lex_string_literal(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<SpannedToken, Diagnostic> {
        // Consume opening quote
        self.advance();

        let mut value = String::new();

        loop {
            match self.peek() {
                None => {
                    return Err(Diagnostic {
                        severity: Severity::Error,
                        summary: "unterminated string literal".to_string(),
                        file: self.file.clone(),
                        line: start_line,
                        col: start_col,
                        explanation: Some(
                            "string literal was opened here but never closed".to_string(),
                        ),
                        suggestion: Some("add a closing '\"'".to_string()),
                    });
                }
                Some('"') => {
                    self.advance(); // consume closing quote
                    return Ok(SpannedToken {
                        token: Token::StringLiteral(value),
                        span: self.make_span_from(start_line, start_col),
                    });
                }
                Some('\\') => {
                    self.advance(); // consume backslash
                    match self.peek() {
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some(c) => {
                            return Err(self.error_at(format!("unknown escape sequence: '\\{c}'")));
                        }
                        None => {
                            return Err(
                                self.error_at("unterminated escape sequence at end of input")
                            );
                        }
                    }
                }
                Some('\n') => {
                    return Err(Diagnostic {
                        severity: Severity::Error,
                        summary: "unterminated string literal (newline in string)".to_string(),
                        file: self.file.clone(),
                        line: start_line,
                        col: start_col,
                        explanation: Some("string literals cannot span multiple lines".to_string()),
                        suggestion: Some(
                            "use \\n for newlines, or close the string before the line break"
                                .to_string(),
                        ),
                    });
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }
    }

    fn lex_line_comment(&mut self) {
        // Consume the two slashes
        self.advance();
        self.advance();
        // Consume until newline or EOF
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn lex_block_comment(&mut self) -> Result<(), Diagnostic> {
        let start_line = self.line;
        let start_col = self.col;
        // Consume /*
        self.advance();
        self.advance();

        loop {
            match self.peek() {
                None => {
                    return Err(Diagnostic {
                        severity: Severity::Error,
                        summary: "unterminated block comment".to_string(),
                        file: self.file.clone(),
                        line: start_line,
                        col: start_col,
                        explanation: Some(
                            "block comment was opened here but never closed".to_string(),
                        ),
                        suggestion: Some("add a closing '*/'".to_string()),
                    });
                }
                Some('*') => {
                    if self.peek_next() == Some('/') {
                        self.advance(); // consume *
                        self.advance(); // consume /
                        return Ok(());
                    }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Character classification
// ---------------------------------------------------------------------------

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Classify an identifier text as a keyword, or return `None` for a plain identifier.
fn classify_keyword(text: &str) -> Option<Keyword> {
    match text {
        // Language keywords
        "namespace" => Some(Keyword::Namespace),
        "uses" => Some(Keyword::Uses),
        "type" => Some(Keyword::Type),
        "plural" => Some(Keyword::Plural),
        "required" => Some(Keyword::Required),
        "optional" => Some(Keyword::Optional),
        // Primitive type names (lexically reserved per DEC-007)
        "string" => Some(Keyword::String),
        "int64" => Some(Keyword::Int64),
        "real64" => Some(Keyword::Real64),
        "bool" => Some(Keyword::Bool),
        "date" => Some(Keyword::Date),
        "time" => Some(Keyword::Time),
        "datetime" => Some(Keyword::DateTime),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
