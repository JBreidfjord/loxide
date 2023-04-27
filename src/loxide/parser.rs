use thiserror::Error;

use super::{
    ast::{Expr, Literal},
    token::Token,
    token_type::TokenType,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("[line {line}] {msg}")]
    Parse { msg: String, line: usize },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, Vec<Error>> {
        self.expression().map_err(|e| vec![e])
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let previous = self.advance();
        match previous.get_token_type() {
            TokenType::False => Ok(Expr::Literal(Literal::Bool(false))),
            TokenType::True => Ok(Expr::Literal(Literal::Bool(true))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenType::Number(v) => Ok(Expr::Literal(Literal::Number(v))),
            TokenType::String(v) => Ok(Expr::Literal(Literal::String(v))),

            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }

            _ => Err(Error::Parse {
                msg: "Expect expression.".to_owned(),
                line: previous.get_line(),
            }),
        }
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        // Discard tokens until we reach a statement boundary
        while !self.is_at_end() {
            if self.previous().get_token_type() == TokenType::Semicolon {
                return;
            }

            match self.peek().get_token_type() {
                TokenType::Class
                | TokenType::Fn
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(Error::Parse {
                msg: message.to_owned(),
                line: self.peek().get_line(),
            })
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().get_token_type() == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].to_owned()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].to_owned()
    }

    fn is_at_end(&self) -> bool {
        self.peek().get_token_type() == TokenType::Eof
    }
}
