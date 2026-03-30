// SPDX-License-Identifier: Apache-2.0
//! Lexer unit tests.

use super::*;
use crate::lexer::token::{Keyword, Token};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn lex_ok(input: &str) -> Vec<Token> {
    let (tokens, diags) = lex(input, "test.pht");
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    tokens.into_iter().map(|st| st.token).collect()
}

fn lex_has_error(input: &str) -> Vec<Diagnostic> {
    let (_, diags) = lex(input, "test.pht");
    assert!(!diags.is_empty(), "expected diagnostics but got none");
    diags
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn keywords() {
    let tokens = lex_ok("namespace uses type plural required optional");
    assert_eq!(
        tokens,
        vec![
            Token::Keyword(Keyword::Namespace),
            Token::Keyword(Keyword::Uses),
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::Plural),
            Token::Keyword(Keyword::Required),
            Token::Keyword(Keyword::Optional),
            Token::Eof,
        ]
    );
}

#[test]
fn primitive_type_keywords() {
    let tokens = lex_ok("string int64 real64 bool date time datetime");
    assert_eq!(
        tokens,
        vec![
            Token::Keyword(Keyword::String),
            Token::Keyword(Keyword::Int64),
            Token::Keyword(Keyword::Real64),
            Token::Keyword(Keyword::Bool),
            Token::Keyword(Keyword::Date),
            Token::Keyword(Keyword::Time),
            Token::Keyword(Keyword::DateTime),
            Token::Eof,
        ]
    );
}

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

#[test]
fn identifiers() {
    let tokens = lex_ok("CSVFile csv_line _private myField123");
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("CSVFile".to_string()),
            Token::Identifier("csv_line".to_string()),
            Token::Identifier("_private".to_string()),
            Token::Identifier("myField123".to_string()),
            Token::Eof,
        ]
    );
}

// ---------------------------------------------------------------------------
// String literals
// ---------------------------------------------------------------------------

#[test]
fn simple_string_literal() {
    let tokens = lex_ok(r#""hello world""#);
    assert_eq!(
        tokens,
        vec![
            Token::StringLiteral("hello world".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn string_escape_sequences() {
    let tokens = lex_ok(r#""\"\\\n\r\t""#);
    assert_eq!(
        tokens,
        vec![
            Token::StringLiteral("\"\\\n\r\t".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn unterminated_string() {
    let diags = lex_has_error(r#""hello"#);
    assert!(diags[0].summary.contains("unterminated"));
}

#[test]
fn newline_in_string() {
    let diags = lex_has_error("\"hello\nworld\"");
    assert!(diags[0].summary.contains("newline"));
}

#[test]
fn unknown_escape() {
    let diags = lex_has_error(r#""\q""#);
    assert!(diags[0].summary.contains("unknown escape"));
}

// ---------------------------------------------------------------------------
// Punctuation
// ---------------------------------------------------------------------------

#[test]
fn punctuation() {
    let tokens = lex_ok(": ; , { } [ ] ( ) @ + * /");
    assert_eq!(
        tokens,
        vec![
            Token::Colon,
            Token::Semicolon,
            Token::Comma,
            Token::LeftBrace,
            Token::RightBrace,
            Token::LeftBracket,
            Token::RightBracket,
            Token::LeftParen,
            Token::RightParen,
            Token::At,
            Token::Plus,
            Token::Star,
            Token::Slash,
            Token::Eof,
        ]
    );
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn line_comment() {
    let tokens = lex_ok("foo // this is a comment\nbar");
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("foo".to_string()),
            Token::Identifier("bar".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn block_comment() {
    let tokens = lex_ok("foo /* block\ncomment */ bar");
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("foo".to_string()),
            Token::Identifier("bar".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn unterminated_block_comment() {
    let diags = lex_has_error("foo /* never closed");
    assert!(diags[0].summary.contains("unterminated block comment"));
}

// ---------------------------------------------------------------------------
// Whitespace
// ---------------------------------------------------------------------------

#[test]
fn whitespace_is_skipped() {
    let tokens = lex_ok("  \t\n  foo  \n  bar  ");
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("foo".to_string()),
            Token::Identifier("bar".to_string()),
            Token::Eof,
        ]
    );
}

// ---------------------------------------------------------------------------
// Span tracking
// ---------------------------------------------------------------------------

#[test]
fn span_positions() {
    let (tokens, _) = lex("foo bar", "test.pht");
    // "foo" starts at col 1, "bar" starts at col 5
    assert_eq!(tokens[0].span.start_line, 1);
    assert_eq!(tokens[0].span.start_col, 1);
    assert_eq!(tokens[0].span.end_col, 3);

    assert_eq!(tokens[1].span.start_line, 1);
    assert_eq!(tokens[1].span.start_col, 5);
    assert_eq!(tokens[1].span.end_col, 7);
}

#[test]
fn span_multiline() {
    let (tokens, _) = lex("foo\nbar", "test.pht");
    assert_eq!(tokens[0].span.start_line, 1);
    assert_eq!(tokens[1].span.start_line, 2);
}

// ---------------------------------------------------------------------------
// Unknown character
// ---------------------------------------------------------------------------

#[test]
fn unknown_character() {
    let diags = lex_has_error("foo $ bar");
    assert!(diags[0].summary.contains("unexpected character"));
}

// ---------------------------------------------------------------------------
// Integration: realistic input
// ---------------------------------------------------------------------------

#[test]
fn csv_phenotype_snippet() {
    let input = r#"namespace aivolution/format/csv;

CSVLine plural CSVLines:
    fields: required CSVFieldValues,
    separator: required string,
    @join(fields, separator)
;"#;
    let (tokens, diags) = lex(input, "csv.pht");
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");

    // Check key tokens are present
    let kinds: Vec<&Token> = tokens.iter().map(|t| &t.token).collect();
    assert!(kinds.contains(&&Token::Keyword(Keyword::Namespace)));
    assert!(kinds.contains(&&Token::Keyword(Keyword::Plural)));
    assert!(kinds.contains(&&Token::Keyword(Keyword::Required)));
    assert!(kinds.contains(&&Token::Keyword(Keyword::String)));
    assert!(kinds.contains(&&Token::At));
    assert!(kinds.contains(&&Token::Slash));
    assert!(kinds.contains(&&Token::Semicolon));
    assert!(kinds.contains(&&Token::Eof));
}

#[test]
fn optional_and_directives_snippet() {
    let input = r#"CSVFile:
    footer: optional CSVLine,
    @ifset(footer) { @eol, @(footer) }
;"#;
    let (tokens, diags) = lex(input, "test.pht");
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");

    let kinds: Vec<&Token> = tokens.iter().map(|t| &t.token).collect();
    assert!(kinds.contains(&&Token::Keyword(Keyword::Optional)));
    assert!(kinds.contains(&&Token::At));
    assert!(kinds.contains(&&Token::LeftBrace));
    assert!(kinds.contains(&&Token::RightBrace));
}
