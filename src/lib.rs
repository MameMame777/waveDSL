pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod preprocessor;
pub mod semantic;
pub mod token;

use std::path::Path;

use error::WaveDslError;

/// Compile WaveDSL source text to WaveDrom JSON.
///
/// If `file_path` is provided, `include` directives are resolved relative
/// to that file's parent directory.
pub fn compile(input: &str, file_path: Option<&Path>) -> Result<serde_json::Value, Vec<WaveDslError>> {
    // Preprocessor: expand includes
    let source = if let Some(path) = file_path {
        let base_dir = path.parent().unwrap_or(Path::new("."));
        preprocessor::expand_includes(input, base_dir).map_err(|e| vec![e])?
    } else {
        input.to_string()
    };

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| vec![e])?;

    let mut parser = parser::Parser::new(tokens);
    let mut program = parser.parse().map_err(|e| vec![e])?;

    semantic::resolve_and_validate(&mut program)?;

    Ok(codegen::generate(&program))
}
