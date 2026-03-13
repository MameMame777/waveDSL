use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords — wave declarations
    Signal,
    Group,
    Repeat,
    Head,
    Foot,
    Config,
    Const,
    Include,

    // Keywords — assertions
    Assert,
    When,
    Then,
    And,
    Or,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Eq,         // =   (keyword-arg assignment)
    EqEq,       // ==  (condition comparison)
    BangEq,     // !=  (condition comparison)
    PoundPound,    // ## (SVA delay)
    LBracketStar,  // [* (consecutive repeat)
    LBracketArrow, // [-> (goto repeat)
    RBracket,      // ]

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
