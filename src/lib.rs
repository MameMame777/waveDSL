pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod token;

use error::WaveDslError;

/// Compile WaveDSL source text to WaveDrom JSON.
pub fn compile(input: &str) -> Result<serde_json::Value, Vec<WaveDslError>> {
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| vec![e])?;

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse().map_err(|e| vec![e])?;

    semantic::validate(&program)?;

    Ok(codegen::generate(&program))
}
