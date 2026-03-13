pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod preprocessor;
pub mod semantic;
pub mod svagen;
pub mod token;

use std::path::Path;

use error::WaveDslError;

/// Compile WaveDSL source text to WaveDrom JSON.
///
/// If `file_path` is provided, `include` directives are resolved relative
/// to that file's parent directory.
pub fn compile(input: &str, file_path: Option<&Path>) -> Result<serde_json::Value, Vec<WaveDslError>> {
    let (json, _sv) = compile_full(input, file_path)?;
    Ok(json)
}

/// Compile WaveDSL source text to both WaveDrom JSON and optional SystemVerilog assertions.
///
/// Returns `(json, sv_text)` where `sv_text` is `Some(...)` when the source
/// contains at least one `assert` block.
pub fn compile_full(
    input: &str,
    file_path: Option<&Path>,
) -> Result<(serde_json::Value, Option<String>), Vec<WaveDslError>> {
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

    let json = codegen::generate(&program);
    let sv   = svagen::generate_sv(&program);
    Ok((json, sv))
}
