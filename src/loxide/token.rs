use std::fmt;

use super::token_type::TokenType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn get_token_type(&self) -> TokenType {
        self.token_type.clone()
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_lexeme(&self) -> String {
        self.lexeme.clone()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type: {:?} | Lexeme: {} | Line: {}",
            self.token_type, self.lexeme, self.line
        )
    }
}
