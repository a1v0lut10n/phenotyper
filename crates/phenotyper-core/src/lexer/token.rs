// SPDX-License-Identifier: Apache-2.0
//! Token and span definitions for the Phenotyper v1 lexer.

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
    /// End column (1-indexed, inclusive).
    pub end_col: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}-{}:{}",
            self.file, self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}

/// A token with its source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// All v1 token kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // --- Identifiers and literals ---
    /// A user-defined identifier (type name, field name, etc.).
    Identifier(String),
    /// A string literal with its decoded (unescaped) value.
    StringLiteral(String),

    // --- Keywords (language + lexically reserved primitive types) ---
    Keyword(Keyword),

    // --- Punctuation ---
    Colon,      // :
    Semicolon,  // ;
    Comma,      // ,
    LeftBrace,  // {
    RightBrace, // }
    LeftBracket,  // [
    RightBracket, // ]
    LeftParen,  // (
    RightParen, // )
    At,         // @
    Plus,       // +
    Star,       // *
    Slash,      // /

    // --- Sentinel ---
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "identifier '{s}'"),
            Token::StringLiteral(s) => write!(f, "string \"{s}\""),
            Token::Keyword(kw) => write!(f, "keyword '{kw}'"),
            Token::Colon => write!(f, "':'"),
            Token::Semicolon => write!(f, "';'"),
            Token::Comma => write!(f, "','"),
            Token::LeftBrace => write!(f, "'{{'"),
            Token::RightBrace => write!(f, "'}}'"),
            Token::LeftBracket => write!(f, "'['"),
            Token::RightBracket => write!(f, "']'"),
            Token::LeftParen => write!(f, "'('"),
            Token::RightParen => write!(f, "')'"),
            Token::At => write!(f, "'@'"),
            Token::Plus => write!(f, "'+'"),
            Token::Star => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Eof => write!(f, "end of file"),
        }
    }
}

/// Keywords in the v1 language.
///
/// This includes both language keywords (`namespace`, `uses`, etc.) and
/// lexically reserved primitive type names (`string`, `int64`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // Language keywords
    Namespace,
    Uses,
    Type,
    Plural,
    Required,
    Optional,

    // Primitive type names (lexically reserved per DEC-007)
    String,
    Int64,
    Real64,
    Bool,
    Date,
    Time,
    DateTime,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Keyword::Namespace => "namespace",
            Keyword::Uses => "uses",
            Keyword::Type => "type",
            Keyword::Plural => "plural",
            Keyword::Required => "required",
            Keyword::Optional => "optional",
            Keyword::String => "string",
            Keyword::Int64 => "int64",
            Keyword::Real64 => "real64",
            Keyword::Bool => "bool",
            Keyword::Date => "date",
            Keyword::Time => "time",
            Keyword::DateTime => "datetime",
        };
        write!(f, "{s}")
    }
}
