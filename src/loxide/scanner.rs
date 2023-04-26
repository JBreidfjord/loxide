use thiserror::Error;

use super::token::Token;
use super::token_type::{TokenType, KEYWORDS};

#[derive(Debug, Error)]
pub enum Error {
    #[error("[line {line}] Invalid UTF-8 character")]
    InvalidUtf8Char { line: usize },

    #[error("[line {line}] Unexpected character `{c}`")]
    UnexpectedCharacter { c: char, line: usize },

    #[error("[line {line}] Unterminated string")]
    UnterminatedString { line: usize },

    #[error(transparent)]
    NumberParse(#[from] std::num::ParseFloatError),
}

pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: Vec<u8>) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            // We are at the beginning of the next lexeme
            self.start = self.current;
            match self
                .scan_token()
                .and_then(|ov| ov.map(|t| self.make_token(t)).transpose())
            {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(error) => errors.push(error),
            }
        }

        // Add the EOF token
        tokens.push(Token::new(TokenType::Eof, String::new(), self.line));

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<Option<TokenType>, Error> {
        match self.advance() {
            // Single character tokens
            b'(' => Ok(Some(TokenType::LeftParen)),
            b')' => Ok(Some(TokenType::RightParen)),
            b'{' => Ok(Some(TokenType::LeftBrace)),
            b'}' => Ok(Some(TokenType::RightBrace)),
            b',' => Ok(Some(TokenType::Comma)),
            b'.' => Ok(Some(TokenType::Dot)),
            b'-' => Ok(Some(TokenType::Minus)),
            b'+' => Ok(Some(TokenType::Plus)),
            b';' => Ok(Some(TokenType::Semicolon)),
            b'*' => Ok(Some(TokenType::Star)),

            // One or two character operators
            b'!' => Ok(Some(if self.match_char(b'=') {
                TokenType::BangEqual
            } else {
                TokenType::Bang
            })),

            b'=' => Ok(Some(if self.match_char(b'=') {
                TokenType::EqualEqual
            } else {
                TokenType::Equal
            })),

            b'<' => Ok(Some(if self.match_char(b'=') {
                TokenType::LessEqual
            } else {
                TokenType::Less
            })),

            b'>' => Ok(Some(if self.match_char(b'=') {
                TokenType::GreaterEqual
            } else {
                TokenType::Greater
            })),

            b'/' => {
                if self.match_char(b'/') {
                    // A comment goes until the end of the line
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(None)
                } else {
                    Ok(Some(TokenType::Slash))
                }
            }

            // Ignore whitespace
            b' ' | b'\r' | b'\t' => Ok(None),
            b'\n' => {
                self.line += 1;
                Ok(None)
            }

            // String literals
            b'"' => self.string().map(Some),

            // Number literals
            c if c.is_ascii_digit() => self.number().map(Some),

            // Identifiers and keywords
            c if c.is_ascii_alphabetic() || c == b'_' => self.identifier().map(Some),

            // Default, unknown character
            c => Err(Error::UnexpectedCharacter {
                c: c as char,
                line: self.line,
            }),
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn make_token(&mut self, token_type: TokenType) -> Result<Token, Error> {
        let text = self.substring(self.start, self.current)?;
        Ok(Token::new(token_type, text, self.line))
    }

    fn match_char(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            return b'\0';
        }
        self.source[self.current + 1]
    }

    fn substring(&self, start: usize, end: usize) -> Result<String, Error> {
        String::from_utf8(self.source[start..end].to_vec())
            .map_err(|_| Error::InvalidUtf8Char { line: self.line })
    }

    fn string(&mut self) -> Result<TokenType, Error> {
        // Seek to the end of the string
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::UnterminatedString { line: self.line });
        }

        // Consume the closing quote
        self.advance();

        // Trim the surrounding quotes
        let value = self.substring(self.start + 1, self.current - 1)?;
        Ok(TokenType::String(value))
    }

    fn number(&mut self) -> Result<TokenType, Error> {
        // Seek to the end of the number
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            // Seek to the end of the fractional part
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = self.substring(self.start, self.current)?.parse::<f64>()?;
        Ok(TokenType::Number(value))
    }

    fn identifier(&mut self) -> Result<TokenType, Error> {
        // Seek to the end of the identifier
        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.advance();
        }

        // Check if the identifier is a reserved keyword
        let text = self.substring(self.start, self.current)?;
        if let Some(token_type) = KEYWORDS.get(&text) {
            Ok(token_type.to_owned())
        } else {
            Ok(TokenType::Identifier(text))
        }
    }
}
