use crate::error::{Span, WaveDslError};
use crate::token::{SpannedToken, Token};

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, WaveDslError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.input.len() {
                tokens.push(SpannedToken::new(Token::Eof, self.span()));
                break;
            }
            let tok = self.next_token()?;
            tokens.push(tok);
        }
        Ok(tokens)
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> u8 {
        let ch = self.input[self.pos];
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
                self.advance();
            }
            // Skip line comments: // ...
            if self.pos + 1 < self.input.len()
                && self.input[self.pos] == b'/'
                && self.input[self.pos + 1] == b'/'
            {
                while self.pos < self.input.len() && self.input[self.pos] != b'\n' {
                    self.advance();
                }
                continue;
            }
            break;
        }
    }

    fn next_token(&mut self) -> Result<SpannedToken, WaveDslError> {
        let span = self.span();
        let ch = self.peek().unwrap();

        match ch {
            b'(' => {
                self.advance();
                Ok(SpannedToken::new(Token::LParen, span))
            }
            b')' => {
                self.advance();
                Ok(SpannedToken::new(Token::RParen, span))
            }
            b'{' => {
                self.advance();
                Ok(SpannedToken::new(Token::LBrace, span))
            }
            b'}' => {
                self.advance();
                Ok(SpannedToken::new(Token::RBrace, span))
            }
            b',' => {
                self.advance();
                Ok(SpannedToken::new(Token::Comma, span))
            }
            b'=' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(SpannedToken::new(Token::EqEq, span))
                } else {
                    Ok(SpannedToken::new(Token::Eq, span))
                }
            }
            b'!' => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(SpannedToken::new(Token::BangEq, span))
                } else {
                    Err(WaveDslError::Lexer {
                        span,
                        message: "unexpected '!'; did you mean '!='?".to_string(),
                    })
                }
            }
            b'#' => {
                self.advance();
                if self.peek() == Some(b'#') {
                    self.advance();
                    Ok(SpannedToken::new(Token::PoundPound, span))
                } else {
                    Err(WaveDslError::Lexer {
                        span,
                        message: "unexpected '#'; did you mean '##'?".to_string(),
                    })
                }
            }
            b'[' => {
                self.advance();
                if self.peek() == Some(b'*') {
                    self.advance();
                    Ok(SpannedToken::new(Token::LBracketStar, span))
                } else if self.peek() == Some(b'-')
                    && self.pos + 1 < self.input.len()
                    && self.input[self.pos + 1] == b'>'
                {
                    self.advance(); // consume '-'
                    self.advance(); // consume '>'
                    Ok(SpannedToken::new(Token::LBracketArrow, span))
                } else {
                    Err(WaveDslError::Lexer {
                        span,
                        message: "unexpected '['; expected '[*' or '[->'.".to_string(),
                    })
                }
            }
            b']' => {
                self.advance();
                Ok(SpannedToken::new(Token::RBracket, span))
            }
            b'"' => self.read_string(span),
            b'0'..=b'9' => self.read_number(span),
            b'$' => self.read_dollar_ident(span),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.read_ident(span),
            _ => Err(WaveDslError::Lexer {
                span,
                message: format!("unexpected character '{}'", ch as char),
            }),
        }
    }

    fn read_string(&mut self, span: Span) -> Result<SpannedToken, WaveDslError> {
        self.advance(); // consume opening "
        let mut s = String::new();
        loop {
            if self.pos >= self.input.len() {
                return Err(WaveDslError::Lexer {
                    span,
                    message: "unterminated string literal".to_string(),
                });
            }
            let ch = self.advance();
            if ch == b'"' {
                break;
            }
            s.push(ch as char);
        }
        Ok(SpannedToken::new(Token::StringLit(s), span))
    }

    fn read_number(&mut self, span: Span) -> Result<SpannedToken, WaveDslError> {
        // Check for hex: 0x...
        if self.peek() == Some(b'0')
            && self.pos + 1 < self.input.len()
            && (self.input[self.pos + 1] == b'x' || self.input[self.pos + 1] == b'X')
        {
            self.advance(); // '0'
            self.advance(); // 'x'
            let start = self.pos;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_hexdigit() {
                self.advance();
            }
            if self.pos == start {
                return Err(WaveDslError::Lexer {
                    span,
                    message: "expected hex digits after 0x".to_string(),
                });
            }
            let hex_str = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
            let value = u64::from_str_radix(hex_str, 16).map_err(|_| WaveDslError::Lexer {
                span,
                message: format!("invalid hex number: 0x{hex_str}"),
            })?;
            return Ok(SpannedToken::new(Token::Number(value), span));
        }

        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.advance();
        }

        // Check for float: digits followed by '.' and another digit
        if self.pos < self.input.len()
            && self.input[self.pos] == b'.'
            && self.pos + 1 < self.input.len()
            && self.input[self.pos + 1].is_ascii_digit()
        {
            self.advance(); // consume '.'
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.advance();
            }
            let float_str = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
            let value = float_str.parse::<f64>().map_err(|_| WaveDslError::Lexer {
                span,
                message: format!("invalid float: {float_str}"),
            })?;
            return Ok(SpannedToken::new(Token::Float(value), span));
        }

        let num_str = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
        let value = num_str.parse::<u64>().map_err(|_| WaveDslError::Lexer {
            span,
            message: format!("invalid number: {num_str}"),
        })?;
        Ok(SpannedToken::new(Token::Number(value), span))
    }

    fn read_dollar_ident(&mut self, span: Span) -> Result<SpannedToken, WaveDslError> {
        self.advance(); // consume '$'
        if self.pos >= self.input.len()
            || !(self.input[self.pos].is_ascii_alphabetic() || self.input[self.pos] == b'_')
        {
            return Err(WaveDslError::Lexer {
                span,
                message: "expected identifier after '$'".to_string(),
            });
        }
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == b'_')
        {
            self.advance();
        }
        let name = std::str::from_utf8(&self.input[start..self.pos])
            .unwrap()
            .to_string();
        Ok(SpannedToken::new(Token::DollarIdent(name), span))
    }

    fn read_ident(&mut self, span: Span) -> Result<SpannedToken, WaveDslError> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == b'_')
        {
            self.advance();
        }
        let ident = std::str::from_utf8(&self.input[start..self.pos]).unwrap().to_string();
        let token = match ident.as_str() {
            "signal"  => Token::Signal,
            "group"   => Token::Group,
            "repeat"  => Token::Repeat,
            "head"    => Token::Head,
            "foot"    => Token::Foot,
            "config"  => Token::Config,
            "const"   => Token::Const,
            "include" => Token::Include,
            "assert"  => Token::Assert,
            "when"    => Token::When,
            "then"    => Token::Then,
            "and"     => Token::And,
            "or"      => Token::Or,
            _ => Token::Ident(ident),
        };
        Ok(SpannedToken::new(token, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_signal() {
        let input = "signal clk clock(8)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 7); // signal, clk, clock, (, 8, ), Eof
        assert_eq!(tokens[0].token, Token::Signal);
        assert_eq!(tokens[1].token, Token::Ident("clk".to_string()));
        assert_eq!(tokens[2].token, Token::Ident("clock".to_string()));
        assert_eq!(tokens[3].token, Token::LParen);
        assert_eq!(tokens[4].token, Token::Number(8));
        assert_eq!(tokens[5].token, Token::RParen);
        assert_eq!(tokens[6].token, Token::Eof);
    }

    #[test]
    fn test_string_literal() {
        let input = r#"data(2, "CMD")"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        // data ( 2 , "CMD" ) Eof
        assert_eq!(tokens[4].token, Token::StringLit("CMD".to_string()));
    }

    #[test]
    fn test_hex_number() {
        let input = "0xFF";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Number(255));
    }

    #[test]
    fn test_comment() {
        let input = "// comment\nsignal clk clock(8)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Signal);
    }

    #[test]
    fn test_keyword_arg() {
        let input = "clock(8, edge=rising)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        // clock ( 8 , edge = rising ) Eof
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[4].token, Token::Ident("edge".to_string()));
        assert_eq!(tokens[5].token, Token::Eq);
        assert_eq!(tokens[6].token, Token::Ident("rising".to_string()));
    }

    #[test]
    fn test_group_braces() {
        let input = r#"group "AXI" { }"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Group);
        assert_eq!(tokens[1].token, Token::StringLit("AXI".to_string()));
        assert_eq!(tokens[2].token, Token::LBrace);
        assert_eq!(tokens[3].token, Token::RBrace);
    }

    #[test]
    fn test_float_literal() {
        let input = "0.5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Float(0.5));
    }

    #[test]
    fn test_float_in_context() {
        let input = "phase=0.5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Ident("phase".to_string()));
        assert_eq!(tokens[1].token, Token::Eq);
        assert_eq!(tokens[2].token, Token::Float(0.5));
    }

    #[test]
    fn test_head_foot_config_keywords() {
        let input = "head foot config";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Head);
        assert_eq!(tokens[1].token, Token::Foot);
        assert_eq!(tokens[2].token, Token::Config);
    }
}
