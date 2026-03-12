use std::fmt;

/// Source location for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WaveDslError {
    #[error("{span}: lexer error: {message}")]
    Lexer { span: Span, message: String },

    #[error("{span}: parse error: {message}")]
    Parser { span: Span, message: String },

    #[error("{span}: semantic error: {message}")]
    Semantic { span: Span, message: String },

    #[error("preprocessor error: {message}")]
    Preprocessor { message: String },
}
