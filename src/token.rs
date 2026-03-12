use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Signal,
    Group,
    Repeat,
    Head,
    Foot,
    Config,
    Const,
    Include,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Eq,

    // Literals
    Number(u64),
    Float(f64),
    StringLit(String),

    // Identifier (signal names, function names, enum values)
    Ident(String),

    // Variable reference: $NAME
    DollarIdent(String),

    Eof,
}

/// Token with source location.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }
}
