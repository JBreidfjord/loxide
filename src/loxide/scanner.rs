use super::token::Token;
use super::token_type::TokenType;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme
            self.start = self.current;
            self.scan_token();
        }

        // Add the EOF token
        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), None, self.line));

        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        todo!()
    }
}
