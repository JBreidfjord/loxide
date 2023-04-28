use thiserror::Error;

use super::{
    ast::{Expr, Literal, Stmt},
    token::Token,
    token_type::TokenType,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("[line {line}] {msg}")]
    Syntax { msg: String, line: usize },
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<Error>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => errors.push(err),
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let result = if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        // Synchronize on error
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expect variable name.")?;

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        // match_token is used so we can consume the token if it matches
        if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if' condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(Error::Syntax {
                    msg: "Invalid assignment target.".to_string(),
                    line: equals.get_line(),
                }),
            }
        } else {
            Ok(expr)
        }
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
            TokenType::Number(n) => Ok(Expr::Literal(Literal::Number(n))),
            TokenType::String(s) => Ok(Expr::Literal(Literal::String(s))),

            TokenType::Identifier(_) => Ok(Expr::Variable(previous)),

            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }

            _ => Err(Error::Syntax {
                msg: "Expect expression.".to_owned(),
                line: previous.get_line(),
            }),
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        // Discard tokens until we reach a statement boundary
        while !self.is_at_end() {
            if self.previous().get_token_type() == TokenType::Semicolon {
                return;
            }

            if matches!(
                self.peek().get_token_type(),
                TokenType::Class
                    | TokenType::Fn
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return
            ) {
                return;
            }

            self.advance();
        }
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance(); // Consume the token
                return true;
            }
        }

        false
    }

    fn consume<S: ToString>(&mut self, token_type: TokenType, message: S) -> Result<Token> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(Error::Syntax {
                msg: message.to_string(),
                line: self.peek().get_line(),
            })
        }
    }

    fn consume_identifier<S: ToString>(&mut self, message: S) -> Result<Token> {
        let error = Error::Syntax {
            msg: message.to_string(),
            line: self.peek().get_line(),
        };
        if self.is_at_end() {
            return Err(error);
        }
        match self.peek().get_token_type() {
            TokenType::Identifier(_) => Ok(self.advance()),
            _ => Err(error),
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
