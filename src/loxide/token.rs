use std::fmt;

use super::token_type::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let literal = match &self.token_type {
            TokenType::String(s) | TokenType::Identifier(s) => s.to_owned(),
            TokenType::Number(n) => n.to_string(),
            _ => String::new(),
        };
        write!(f, "{:?} {} {}", self.token_type, self.lexeme, literal)
    }
}
