use crate::ast::*;
use crate::error::{Span, WaveDslError};

const RESERVED_WORDS: &[&str] = &["clock", "high", "low", "data", "x", "z", "gap", "repeat"];
const KNOWN_FUNCTIONS: &[&str] = &["clock", "high", "low", "data", "x", "z", "gap"];

pub fn validate(program: &Program) -> Result<(), Vec<WaveDslError>> {
    let mut errors = Vec::new();
    for stmt in &program.statements {
        validate_statement(stmt, 0, &mut errors);
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
}
