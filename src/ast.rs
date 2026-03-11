use crate::error::Span;

/// Top-level program: a list of statements.
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A statement is either a signal declaration or a group.
#[derive(Debug, Clone)]
pub enum Statement {
    Signal {
        name: String,
        sequence: Vec<WaveExpr>,
        span: Span,
    },
    Group {
        name: Option<String>,
        statements: Vec<Statement>,
        span: Span,
    },
}

/// A wave expression: a built-in function call or repeat.
#[derive(Debug, Clone)]
pub enum WaveExpr {
    Call {
        name: String,
        args: Vec<Arg>,
        span: Span,
    },
    Repeat {
        count: u64,
        sequence: Vec<WaveExpr>,
        span: Span,
    },
}

/// Function argument: positional or keyword.
#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Value, Span),
    Keyword(String, Value, Span),
}

/// Argument value.
#[derive(Debug, Clone)]
pub enum Value {
    Number(u64),
    Str(String),
    Enum(String),
}
