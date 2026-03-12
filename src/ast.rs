use crate::error::Span;

/// Top-level program: statements plus optional head/foot/config blocks.
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub head: Option<Vec<KeyValue>>,
    pub foot: Option<Vec<KeyValue>>,
    pub config: Option<Vec<KeyValue>>,
}

/// A key-value pair used in head/foot/config blocks.
#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: Value,
    pub span: Span,
}

/// A signal-level attribute (e.g. period=2, phase=0.5).
#[derive(Debug, Clone)]
pub struct SignalAttr {
    pub name: String,
    pub value: Value,
    pub span: Span,
}

/// A statement is either a signal declaration or a group.
#[derive(Debug, Clone)]
pub enum Statement {
    Signal {
        name: String,
        sequence: Vec<WaveExpr>,
        attrs: Vec<SignalAttr>,
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
    Float(f64),
    Str(String),
    Enum(String),
}
