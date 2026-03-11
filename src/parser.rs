use crate::ast::*;
use crate::error::{Span, WaveDslError};
use crate::token::{SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, WaveDslError> {
        let mut statements = Vec::new();
        while !self.at_eof() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn advance(&mut self) -> &SpannedToken {
        let tok = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<Span, WaveDslError> {
        let span = self.span();
        if self.peek() == expected {
            self.advance();
            Ok(span)
        } else {
            Err(WaveDslError::Parser {
                span,
                message: format!("expected {:?}, found {:?}", expected, self.peek()),
            })
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, WaveDslError> {
        match self.peek() {
            Token::Signal => self.parse_signal(),
            Token::Group => self.parse_group(),
            _ => Err(WaveDslError::Parser {
                span: self.span(),
                message: format!(
                    "expected 'signal' or 'group', found {:?}",
                    self.peek()
                ),
            }),
        }
    }

    fn parse_signal(&mut self) -> Result<Statement, WaveDslError> {
        let span = self.span();
        self.advance(); // consume 'signal'

        let name = self.expect_ident()?;
        let sequence = self.parse_sequence()?;

        Ok(Statement::Signal {
            name,
            sequence,
            span,
        })
    }

    fn parse_group(&mut self) -> Result<Statement, WaveDslError> {
        let span = self.span();
        self.advance(); // consume 'group'

        // Optional group name (string literal)
        let name = if let Token::StringLit(_) = self.peek() {
            if let Token::StringLit(s) = &self.advance().token.clone() {
                Some(s.clone())
            } else {
                unreachable!()
            }
        } else {
            None
        };

        self.expect(&Token::LBrace)?;

        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        self.expect(&Token::RBrace)?;

        Ok(Statement::Group {
            name,
            statements,
            span,
        })
    }

    /// Parse one or more wave expressions (the sequence after a signal name).
    fn parse_sequence(&mut self) -> Result<Vec<WaveExpr>, WaveDslError> {
        let mut exprs = Vec::new();
        // A sequence continues as long as the next token is an Ident (function call)
        // or the Repeat keyword.
        while self.is_wave_expr_start() {
            exprs.push(self.parse_wave_expr()?);
        }
        if exprs.is_empty() {
            return Err(WaveDslError::Parser {
                span: self.span(),
                message: "expected at least one wave expression".to_string(),
            });
        }
        Ok(exprs)
    }

    fn is_wave_expr_start(&self) -> bool {
        matches!(self.peek(), Token::Ident(_) | Token::Repeat)
    }

    fn parse_wave_expr(&mut self) -> Result<WaveExpr, WaveDslError> {
        if matches!(self.peek(), Token::Repeat) {
            self.parse_repeat()
        } else {
            self.parse_basic_call()
        }
    }

    fn parse_basic_call(&mut self) -> Result<WaveExpr, WaveDslError> {
        let span = self.span();
        let name = self.expect_ident()?;

        self.expect(&Token::LParen)?;
        let args = self.parse_arg_list()?;
        self.expect(&Token::RParen)?;

        Ok(WaveExpr::Call { name, args, span })
    }

    fn parse_repeat(&mut self) -> Result<WaveExpr, WaveDslError> {
        let span = self.span();
        self.advance(); // consume 'repeat'

        self.expect(&Token::LParen)?;

        // First arg: count (number)
        let count = self.expect_number()?;

        self.expect(&Token::Comma)?;

        // Remaining args: one or more wave_expr
        let mut sequence = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            sequence.push(self.parse_wave_expr()?);
        }
        if sequence.is_empty() {
            return Err(WaveDslError::Parser {
                span,
                message: "repeat requires at least one wave expression".to_string(),
            });
        }

        self.expect(&Token::RParen)?;

        Ok(WaveExpr::Repeat {
            count,
            sequence,
            span,
        })
    }

    /// Parse comma-separated argument list (positional then keyword).
    fn parse_arg_list(&mut self) -> Result<Vec<Arg>, WaveDslError> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(args);
        }

        args.push(self.parse_arg()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance(); // consume ','
            args.push(self.parse_arg()?);
        }
        Ok(args)
    }

    fn parse_arg(&mut self) -> Result<Arg, WaveDslError> {
        let span = self.span();
        // Check if this is a keyword argument: ident = value
        if let Token::Ident(_) = self.peek() {
            // Look ahead for '='
            if self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].token, Token::Eq)
            {
                let key = self.expect_ident()?;
                self.advance(); // consume '='
                let value = self.parse_value()?;
                return Ok(Arg::Keyword(key, value, span));
            }
        }
        // Positional argument
        let value = self.parse_value()?;
        Ok(Arg::Positional(value, span))
    }

    fn parse_value(&mut self) -> Result<Value, WaveDslError> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Value::Number(n))
            }
            Token::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Value::Str(s))
            }
            Token::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(Value::Enum(s))
            }
            _ => Err(WaveDslError::Parser {
                span: self.span(),
                message: format!("expected value, found {:?}", self.peek()),
            }),
        }
    }

    fn expect_ident(&mut self) -> Result<String, WaveDslError> {
        let span = self.span();
        match self.peek().clone() {
            Token::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(WaveDslError::Parser {
                span,
                message: format!("expected identifier, found {:?}", self.peek()),
            }),
        }
    }

    fn expect_number(&mut self) -> Result<u64, WaveDslError> {
        let span = self.span();
        match self.peek() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(n)
            }
            _ => Err(WaveDslError::Parser {
                span,
                message: format!("expected number, found {:?}", self.peek()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_str(input: &str) -> Result<Program, WaveDslError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_simple_signal() {
        let prog = parse_str("signal clk clock(8)").unwrap();
        assert_eq!(prog.statements.len(), 1);
        if let Statement::Signal { name, sequence, .. } = &prog.statements[0] {
            assert_eq!(name, "clk");
            assert_eq!(sequence.len(), 1);
        } else {
            panic!("expected Signal");
        }
    }

    #[test]
    fn test_multiple_exprs() {
        let prog = parse_str(r#"signal mosi x(1) data(2, "CMD") x(1)"#).unwrap();
        if let Statement::Signal { sequence, .. } = &prog.statements[0] {
            assert_eq!(sequence.len(), 3);
        } else {
            panic!("expected Signal");
        }
    }

    #[test]
    fn test_repeat() {
        let prog =
            parse_str(r#"signal burst x(1) repeat(4, data(2, "0xAB") data(2, "0xCD")) x(1)"#)
                .unwrap();
        if let Statement::Signal { sequence, .. } = &prog.statements[0] {
            assert_eq!(sequence.len(), 3);
            if let WaveExpr::Repeat {
                count, sequence, ..
            } = &sequence[1]
            {
                assert_eq!(*count, 4);
                assert_eq!(sequence.len(), 2);
            } else {
                panic!("expected Repeat");
            }
        } else {
            panic!("expected Signal");
        }
    }

    #[test]
    fn test_group() {
        let prog = parse_str(
            r#"group "AXI" {
                signal clk clock(8)
            }"#,
        )
        .unwrap();
        if let Statement::Group {
            name, statements, ..
        } = &prog.statements[0]
        {
            assert_eq!(name.as_deref(), Some("AXI"));
            assert_eq!(statements.len(), 1);
        } else {
            panic!("expected Group");
        }
    }

    #[test]
    fn test_keyword_arg() {
        let prog = parse_str("signal clk clock(8, edge=falling)").unwrap();
        if let Statement::Signal { sequence, .. } = &prog.statements[0] {
            if let WaveExpr::Call { args, .. } = &sequence[0] {
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[1], Arg::Keyword(k, Value::Enum(v), _) if k == "edge" && v == "falling"));
            } else {
                panic!("expected Call");
            }
        } else {
            panic!("expected Signal");
        }
    }
}
