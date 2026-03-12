use std::collections::HashMap;

use crate::ast::*;
use crate::error::{Span, WaveDslError};

const RESERVED_WORDS: &[&str] = &["clock", "high", "low", "data", "x", "z", "gap", "repeat"];
const KNOWN_FUNCTIONS: &[&str] = &["clock", "high", "low", "data", "x", "z", "gap"];
const VALID_SIGNAL_ATTRS: &[&str] = &["period", "phase"];
const VALID_HEAD_FOOT_KEYS: &[&str] = &["text", "tick", "tock", "every"];
const VALID_CONFIG_KEYS: &[&str] = &["hscale"];

/// Resolve constants and then validate the program.
pub fn resolve_and_validate(program: &mut Program) -> Result<(), Vec<WaveDslError>> {
    resolve_constants(program)?;
    validate(program)
}

pub fn validate(program: &Program) -> Result<(), Vec<WaveDslError>> {
    let mut errors = Vec::new();
    for stmt in &program.statements {
        validate_statement(stmt, 0, &mut errors);
    }
    if let Some(head) = &program.head {
        validate_head_foot("head", head, &mut errors);
    }
    if let Some(foot) = &program.foot {
        validate_head_foot("foot", foot, &mut errors);
    }
    if let Some(config) = &program.config {
        validate_config(config, &mut errors);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_statement(stmt: &Statement, group_depth: usize, errors: &mut Vec<WaveDslError>) {
    match stmt {
        Statement::Signal {
            name,
            sequence,
            attrs,
            span,
        } => {
            // Signal name must not be a reserved word
            if RESERVED_WORDS.contains(&name.as_str()) {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: format!("'{}' is a reserved word and cannot be used as a signal name", name),
                });
            }
            for expr in sequence {
                validate_wave_expr(expr, errors);
            }
            for attr in attrs {
                validate_signal_attr(attr, errors);
            }
        }
        Statement::Group {
            statements, span, ..
        } => {
            let new_depth = group_depth + 1;
            if new_depth > 2 {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: "group nesting depth exceeds maximum of 2".to_string(),
                });
            }
            for stmt in statements {
                validate_statement(stmt, new_depth, errors);
            }
        }
        Statement::ConstDecl { .. } => {
            // Already resolved; nothing to validate here.
        }
    }
}

fn validate_wave_expr(expr: &WaveExpr, errors: &mut Vec<WaveDslError>) {
    match expr {
        WaveExpr::Call { name, args, span } => {
            // Check known function name
            if !KNOWN_FUNCTIONS.contains(&name.as_str()) {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: format!("unknown function '{}'", name),
                });
                return;
            }
            validate_function_args(name, args, *span, errors);
        }
        WaveExpr::Repeat {
            count,
            sequence,
            span,
        } => {
            if *count < 1 {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: "repeat count must be >= 1".to_string(),
                });
            }
            for expr in sequence {
                validate_wave_expr(expr, errors);
            }
        }
    }
}

fn validate_function_args(
    name: &str,
    args: &[Arg],
    span: Span,
    errors: &mut Vec<WaveDslError>,
) {
    match name {
        "clock" => {
            // clock(n, edge=rising)
            let n = extract_first_number(args);
            validate_n(n, span, errors);
            // Check edge keyword if present
            for arg in args {
                if let Arg::Keyword(key, value, kspan) = arg {
                    if key == "edge" {
                        if let Value::Enum(v) = value {
                            if v != "rising" && v != "falling" {
                                errors.push(WaveDslError::Semantic {
                                    span: *kspan,
                                    message: format!(
                                        "edge must be 'rising' or 'falling', got '{}'", v
                                    ),
                                });
                            }
                        } else {
                            errors.push(WaveDslError::Semantic {
                                span: *kspan,
                                message: "edge value must be 'rising' or 'falling'".to_string(),
                            });
                        }
                    } else {
                        errors.push(WaveDslError::Semantic {
                            span: *kspan,
                            message: format!("unknown keyword argument '{}' for clock", key),
                        });
                    }
                }
            }
        }
        "high" | "low" | "x" | "z" => {
            // func(n)
            let n = extract_first_number(args);
            validate_n(n, span, errors);
            check_no_keywords(name, args, errors);
        }
        "data" => {
            // data(n, label, color=1)
            let n = extract_first_number(args);
            validate_n(n, span, errors);
            // Check color keyword if present
            for arg in args {
                if let Arg::Keyword(key, value, kspan) = arg {
                    if key == "color" {
                        if let Value::Number(c) = value {
                            if *c < 1 || *c > 5 {
                                errors.push(WaveDslError::Semantic {
                                    span: *kspan,
                                    message: format!(
                                        "color must be 1-5, got {}", c
                                    ),
                                });
                            }
                        } else {
                            errors.push(WaveDslError::Semantic {
                                span: *kspan,
                                message: "color value must be a number 1-5".to_string(),
                            });
                        }
                    } else {
                        errors.push(WaveDslError::Semantic {
                            span: *kspan,
                            message: format!("unknown keyword argument '{}' for data", key),
                        });
                    }
                }
            }
        }
        "gap" => {
            // gap() — no arguments
            let positional_count = args.iter().filter(|a| matches!(a, Arg::Positional(..))).count();
            if positional_count > 0 {
                errors.push(WaveDslError::Semantic {
                    span,
                    message: "gap() takes no arguments".to_string(),
                });
            }
            check_no_keywords(name, args, errors);
        }
        _ => {}
    }
}

fn extract_first_number(args: &[Arg]) -> Option<u64> {
    for arg in args {
        if let Arg::Positional(Value::Number(n), _) = arg {
            return Some(*n);
        }
    }
    None
}

fn validate_n(n: Option<u64>, span: Span, errors: &mut Vec<WaveDslError>) {
    match n {
        Some(0) => {
            errors.push(WaveDslError::Semantic {
                span,
                message: "n must be >= 1".to_string(),
            });
        }
        None => {
            errors.push(WaveDslError::Semantic {
                span,
                message: "missing required argument n".to_string(),
            });
        }
        _ => {}
    }
}

fn check_no_keywords(name: &str, args: &[Arg], errors: &mut Vec<WaveDslError>) {
    for arg in args {
        if let Arg::Keyword(key, _, kspan) = arg {
            errors.push(WaveDslError::Semantic {
                span: *kspan,
                message: format!("'{}' does not accept keyword argument '{}'", name, key),
            });
        }
    }
}

fn validate_signal_attr(attr: &SignalAttr, errors: &mut Vec<WaveDslError>) {
    if !VALID_SIGNAL_ATTRS.contains(&attr.name.as_str()) {
        errors.push(WaveDslError::Semantic {
            span: attr.span,
            message: format!(
                "unknown signal attribute '{}'; expected 'period' or 'phase'",
                attr.name
            ),
        });
        return;
    }
    if !matches!(&attr.value, Value::Number(_) | Value::Float(_)) {
        errors.push(WaveDslError::Semantic {
            span: attr.span,
            message: format!("signal attribute '{}' must be a number", attr.name),
        });
    }
}

fn validate_head_foot(
    block_name: &str,
    pairs: &[KeyValue],
    errors: &mut Vec<WaveDslError>,
) {
    for kv in pairs {
        if !VALID_HEAD_FOOT_KEYS.contains(&kv.key.as_str()) {
            errors.push(WaveDslError::Semantic {
                span: kv.span,
                message: format!(
                    "unknown {} key '{}'; expected one of: text, tick, tock, every",
                    block_name, kv.key
                ),
            });
            continue;
        }
        match kv.key.as_str() {
            "text" => {
                if !matches!(&kv.value, Value::Str(_)) {
                    errors.push(WaveDslError::Semantic {
                        span: kv.span,
                        message: format!("{}.text must be a string", block_name),
                    });
                }
            }
            "tick" | "tock" | "every" => {
                if !matches!(&kv.value, Value::Number(_) | Value::Float(_)) {
                    errors.push(WaveDslError::Semantic {
                        span: kv.span,
                        message: format!("{}.{} must be a number", block_name, kv.key),
                    });
                }
            }
            _ => {}
        }
    }
}

fn validate_config(pairs: &[KeyValue], errors: &mut Vec<WaveDslError>) {
    for kv in pairs {
        if !VALID_CONFIG_KEYS.contains(&kv.key.as_str()) {
            errors.push(WaveDslError::Semantic {
                span: kv.span,
                message: format!("unknown config key '{}'; expected 'hscale'", kv.key),
            });
            continue;
        }
        if kv.key == "hscale" && !matches!(&kv.value, Value::Number(_)) {
            errors.push(WaveDslError::Semantic {
                span: kv.span,
                message: "config.hscale must be an integer".to_string(),
            });
        }
    }
}

// --- Constant resolution ---

const RESERVED_CONST_NAMES: &[&str] = &[
    "signal", "group", "repeat", "head", "foot", "config", "const", "include",
    "clock", "high", "low", "data", "x", "z", "gap",
    "rising", "falling",
];

/// Resolve all `const` declarations and substitute `$NAME` references.
///
/// Constants are resolved in declaration order (no forward references).
/// After resolution, `VarRef` nodes are replaced with the resolved `Value`.
fn resolve_constants(program: &mut Program) -> Result<(), Vec<WaveDslError>> {
    let mut errors = Vec::new();
    let mut table: HashMap<String, Value> = HashMap::new();

    // First pass: collect const declarations in order
    for stmt in &program.statements {
        if let Statement::ConstDecl { name, value, span } = stmt {
            if RESERVED_CONST_NAMES.contains(&name.as_str()) {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: format!("'{}' is reserved and cannot be used as a constant name", name),
                });
                continue;
            }
            if table.contains_key(name) {
                errors.push(WaveDslError::Semantic {
                    span: *span,
                    message: format!("constant '{}' is already defined", name),
                });
                continue;
            }
            // Resolve the value itself (it may reference earlier constants)
            match resolve_value(value, &table, *span) {
                Ok(resolved) => {
                    table.insert(name.clone(), resolved);
                }
                Err(e) => errors.push(e),
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Second pass: substitute VarRef in all statements, attrs, head/foot/config
    for stmt in &mut program.statements {
        resolve_in_statement(stmt, &table, &mut errors);
    }
    if let Some(head) = &mut program.head {
        resolve_in_kv_pairs(head, &table, &mut errors);
    }
    if let Some(foot) = &mut program.foot {
        resolve_in_kv_pairs(foot, &table, &mut errors);
    }
    if let Some(config) = &mut program.config {
        resolve_in_kv_pairs(config, &table, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_value(
    value: &Value,
    table: &HashMap<String, Value>,
    span: Span,
) -> Result<Value, WaveDslError> {
    match value {
        Value::VarRef(name) => {
            table.get(name).cloned().ok_or(WaveDslError::Semantic {
                span,
                message: format!("undefined constant '${}'", name),
            })
        }
        other => Ok(other.clone()),
    }
}

fn resolve_in_statement(
    stmt: &mut Statement,
    table: &HashMap<String, Value>,
    errors: &mut Vec<WaveDslError>,
) {
    match stmt {
        Statement::Signal {
            sequence, attrs, ..
        } => {
            for expr in sequence {
                resolve_in_wave_expr(expr, table, errors);
            }
            for attr in attrs {
                resolve_in_value(&mut attr.value, table, attr.span, errors);
            }
        }
        Statement::Group { statements, .. } => {
            for s in statements {
                resolve_in_statement(s, table, errors);
            }
        }
        Statement::ConstDecl { .. } => {}
    }
}

fn resolve_in_wave_expr(
    expr: &mut WaveExpr,
    table: &HashMap<String, Value>,
    errors: &mut Vec<WaveDslError>,
) {
    match expr {
        WaveExpr::Call { args, span, .. } => {
            for arg in args {
                let (val, arg_span) = match arg {
                    Arg::Positional(v, s) => (v, *s),
                    Arg::Keyword(_, v, s) => (v, *s),
                };
                resolve_in_value(val, table, arg_span, errors);
            }
            // Also check the span for unresolved refs at call level
            let _ = span;
        }
        WaveExpr::Repeat { sequence, .. } => {
            for e in sequence {
                resolve_in_wave_expr(e, table, errors);
            }
        }
    }
}

fn resolve_in_value(
    value: &mut Value,
    table: &HashMap<String, Value>,
    span: Span,
    errors: &mut Vec<WaveDslError>,
) {
    if let Value::VarRef(name) = value {
        match table.get(name.as_str()) {
            Some(resolved) => *value = resolved.clone(),
            None => errors.push(WaveDslError::Semantic {
                span,
                message: format!("undefined constant '${}'", name),
            }),
        }
    }
}

fn resolve_in_kv_pairs(
    pairs: &mut [KeyValue],
    table: &HashMap<String, Value>,
    errors: &mut Vec<WaveDslError>,
) {
    for kv in pairs {
        resolve_in_value(&mut kv.value, table, kv.span, errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn validate_str(input: &str) -> Result<(), Vec<WaveDslError>> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        validate(&program)
    }

    #[test]
    fn test_valid_simple() {
        assert!(validate_str("signal clk clock(8)").is_ok());
    }

    #[test]
    fn test_reserved_signal_name() {
        let result = validate_str("signal clock clock(8)");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("reserved word"));
    }

    #[test]
    fn test_n_zero() {
        let result = validate_str("signal clk clock(0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_group_depth_exceeded() {
        let result = validate_str(
            r#"group "A" { group "B" { group "C" { signal clk clock(1) } } }"#,
        );
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("nesting depth"));
    }

    #[test]
    fn test_invalid_edge() {
        let result = validate_str("signal clk clock(8, edge=both)");
        assert!(result.is_err());
    }

    #[test]
    fn test_color_out_of_range() {
        let result = validate_str(r#"signal d data(2, "X", color=6)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_function() {
        let result = validate_str("signal a foo(1)");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("unknown function"));
    }

    #[test]
    fn test_valid_signal_attr() {
        assert!(validate_str("signal clk clock(8) period=2").is_ok());
        assert!(validate_str("signal cmd x(1) phase=0.5").is_ok());
    }

    #[test]
    fn test_invalid_signal_attr_name() {
        let result = validate_str("signal clk clock(8) foo=1");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("unknown signal attribute"));
    }

    #[test]
    fn test_signal_attr_non_numeric() {
        let result = validate_str(r#"signal clk clock(8) period="fast""#);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("must be a number"));
    }

    #[test]
    fn test_valid_head() {
        assert!(validate_str(r#"head { text="title" tick=0 every=2 }"#).is_ok());
    }

    #[test]
    fn test_invalid_head_key() {
        let result = validate_str(r#"head { color=1 }"#);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("unknown head key"));
    }

    #[test]
    fn test_head_text_must_be_string() {
        let result = validate_str("head { text=42 }");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("must be a string"));
    }

    #[test]
    fn test_valid_config() {
        assert!(validate_str("config { hscale=2 }").is_ok());
    }

    #[test]
    fn test_invalid_config_key() {
        let result = validate_str("config { zoom=1 }");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_hscale_must_be_integer() {
        let result = validate_str("config { hscale=1.5 }");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].to_string().contains("must be an integer"));
    }
}
