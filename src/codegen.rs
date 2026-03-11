use crate::ast::*;
use serde_json::{json, Value as JsonValue};

/// Generate WaveDrom JSON from a validated AST.
pub fn generate(program: &Program) -> JsonValue {
    let signals = generate_statements(&program.statements);
    json!({ "signal": signals })
}

fn generate_statements(statements: &[Statement]) -> Vec<JsonValue> {
    let mut result = Vec::new();
    for stmt in statements {
        match stmt {
            Statement::Signal {
                name, sequence, ..
            } => {
                result.push(generate_signal(name, sequence));
            }
            Statement::Group {
                name, statements, ..
            } => {
                result.push(generate_group(name.as_deref(), statements));
            }
        }
    }
    result
}

fn generate_signal(name: &str, sequence: &[WaveExpr]) -> JsonValue {
    let mut wave = String::new();
    let mut data_labels: Vec<String> = Vec::new();

    for expr in sequence {
        generate_wave_expr(expr, &mut wave, &mut data_labels);
    }

    if data_labels.is_empty() {
        json!({ "name": name, "wave": wave })
    } else {
        json!({ "name": name, "wave": wave, "data": data_labels })
    }
}

fn generate_group(name: Option<&str>, statements: &[Statement]) -> JsonValue {
    let group_name = name.unwrap_or("");
    let mut arr: Vec<JsonValue> = vec![json!(group_name)];
    arr.extend(generate_statements(statements));
    JsonValue::Array(arr)
}

fn generate_wave_expr(expr: &WaveExpr, wave: &mut String, data: &mut Vec<String>) {
    match expr {
        WaveExpr::Call { name, args, .. } => {
            generate_call(name, args, wave, data);
        }
        WaveExpr::Repeat {
            count, sequence, ..
        } => {
            // Expand the sequence, then repeat it count times
            let mut sub_wave = String::new();
            let mut sub_data: Vec<String> = Vec::new();
            for expr in sequence {
                generate_wave_expr(expr, &mut sub_wave, &mut sub_data);
            }
            for _ in 0..*count {
                wave.push_str(&sub_wave);
                data.extend(sub_data.iter().cloned());
            }
        }
    }
}

fn generate_call(name: &str, args: &[Arg], wave: &mut String, data: &mut Vec<String>) {
    match name {
        "clock" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            let edge = get_keyword_enum(args, "edge").unwrap_or_else(|| "rising".to_string());
            let ch = if edge == "falling" { 'N' } else { 'P' };
            // Clock: each slot is independent (no '.' continuation)
            for _ in 0..n {
                wave.push(ch);
            }
        }
        "high" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            wave.push('1');
            for _ in 1..n {
                wave.push('.');
            }
        }
        "low" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            wave.push('0');
            for _ in 1..n {
                wave.push('.');
            }
        }
        "data" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            let label = get_positional_string(args, 1).unwrap_or_default();
            let color = get_keyword_number(args, "color").unwrap_or(1);

            let token = match color {
                1 => '=',
                2 => '2',
                3 => '3',
                4 => '4',
                5 => '5',
                _ => '=',
            };

            wave.push(token);
            for _ in 1..n {
                wave.push('.');
            }
            data.push(label);
        }
        "x" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            wave.push('x');
            for _ in 1..n {
                wave.push('.');
            }
        }
        "z" => {
            let n = get_positional_number(args, 0).unwrap_or(1) as usize;
            wave.push('z');
            for _ in 1..n {
                wave.push('.');
            }
        }
        "gap" => {
            wave.push('|');
        }
        _ => {}
    }
}

/// Get the i-th positional argument as a number.
fn get_positional_number(args: &[Arg], index: usize) -> Option<u64> {
    let mut pos_idx = 0;
    for arg in args {
        if let Arg::Positional(Value::Number(n), _) = arg {
            if pos_idx == index {
                return Some(*n);
            }
            pos_idx += 1;
        } else if matches!(arg, Arg::Positional(..)) {
            pos_idx += 1;
        }
    }
    None
}

/// Get the i-th positional argument as a string.
fn get_positional_string(args: &[Arg], index: usize) -> Option<String> {
    let mut pos_idx = 0;
    for arg in args {
        match arg {
            Arg::Positional(Value::Str(s), _) => {
                if pos_idx == index {
                    return Some(s.clone());
                }
                pos_idx += 1;
            }
            Arg::Positional(..) => {
                pos_idx += 1;
            }
            _ => {}
        }
    }
    None
}

/// Get a keyword argument value as a number.
fn get_keyword_number(args: &[Arg], key: &str) -> Option<u64> {
    for arg in args {
        if let Arg::Keyword(k, Value::Number(n), _) = arg {
            if k == key {
                return Some(*n);
            }
        }
    }
    None
}

/// Get a keyword argument value as an enum string.
fn get_keyword_enum(args: &[Arg], key: &str) -> Option<String> {
    for arg in args {
        if let Arg::Keyword(k, Value::Enum(v), _) = arg {
            if k == key {
                return Some(v.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(input: &str) -> JsonValue {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        generate(&program)
    }

    #[test]
    fn test_clock() {
        let result = compile("signal clk clock(4)");
        assert_eq!(result["signal"][0]["wave"], "PPPP");
    }

    #[test]
    fn test_falling_clock() {
        let result = compile("signal clk clock(4, edge=falling)");
        assert_eq!(result["signal"][0]["wave"], "NNNN");
    }

    #[test]
    fn test_high_low() {
        let result = compile("signal s high(1) low(3) high(1)");
        assert_eq!(result["signal"][0]["wave"], "10..1");
    }

    #[test]
    fn test_data_with_label() {
        let result = compile(r#"signal d x(1) data(2, "CMD") x(1)"#);
        assert_eq!(result["signal"][0]["wave"], "x=.x");
        assert_eq!(result["signal"][0]["data"], json!(["CMD"]));
    }

    #[test]
    fn test_data_with_color() {
        let result = compile(r#"signal d data(2, "X", color=3)"#);
        assert_eq!(result["signal"][0]["wave"], "3.");
    }

    #[test]
    fn test_gap() {
        let result = compile("signal s high(2) gap() low(2)");
        assert_eq!(result["signal"][0]["wave"], "1.|0.");
    }

    #[test]
    fn test_repeat() {
        let result = compile("signal s repeat(3, high(2))");
        assert_eq!(result["signal"][0]["wave"], "1.1.1.");
    }

    #[test]
    fn test_repeat_with_data() {
        let result = compile(r#"signal s repeat(2, data(1, "A") data(1, "B"))"#);
        assert_eq!(result["signal"][0]["wave"], "====");
        assert_eq!(result["signal"][0]["data"], json!(["A", "B", "A", "B"]));
    }

    #[test]
    fn test_spec_complete_example() {
        let input = r#"signal sclk  clock(8)
signal mosi  x(1) data(2, "CMD") data(4, "DATA") x(1)
signal cs_n  high(1) low(6) high(1)"#;
        let result = compile(input);
        assert_eq!(result["signal"][0]["name"], "sclk");
        assert_eq!(result["signal"][0]["wave"], "PPPPPPPP");
        assert_eq!(result["signal"][1]["name"], "mosi");
        assert_eq!(result["signal"][1]["wave"], "x=.=...x");
        assert_eq!(result["signal"][1]["data"], json!(["CMD", "DATA"]));
        assert_eq!(result["signal"][2]["name"], "cs_n");
        assert_eq!(result["signal"][2]["wave"], "10.....1");
    }

    #[test]
    fn test_group() {
        let input = r#"group "AXI" {
            signal clk clock(4)
        }"#;
        let result = compile(input);
        let group = &result["signal"][0];
        assert_eq!(group[0], "AXI");
        assert_eq!(group[1]["name"], "clk");
        assert_eq!(group[1]["wave"], "PPPP");
    }

    #[test]
    fn test_unnamed_group() {
        let input = r#"group {
            signal clk clock(4)
        }"#;
        let result = compile(input);
        let group = &result["signal"][0];
        assert_eq!(group[0], "");
    }

    #[test]
    fn test_z() {
        let result = compile("signal s z(3)");
        assert_eq!(result["signal"][0]["wave"], "z..");
    }

    #[test]
    fn test_x() {
        let result = compile("signal s x(3)");
        assert_eq!(result["signal"][0]["wave"], "x..");
    }
}
