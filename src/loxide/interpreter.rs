use thiserror::Error;

use super::{
    ast::{Expr, Literal, Visitor},
    token_type::TokenType,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error converting literal {literal}.")]
    LiteralConversion { literal: Literal },

    #[error("Unsupported unary operator `{operator}` on type {}.", .value.type_of())]
    UnsupportedUnary {
        operator: TokenType,
        value: ExprReturn,
    },

    #[error(
        "Unsupported binary operator `{operator}` on types {} and {}",
        .left.type_of(),
        .right.type_of()
    )]
    UnsupportedBinary {
        operator: TokenType,
        left: ExprReturn,
        right: ExprReturn,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Interpreter;

#[derive(Debug)]
pub enum ExprReturn {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

impl ExprReturn {
    fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Bool(false))
    }

    fn type_of(&self) -> String {
        match self {
            Self::Nil => String::from("Nil"),
            Self::Number(_) => String::from("Number"),
            Self::Bool(_) => String::from("Bool"),
            Self::String(_) => String::from("String"),
        }
    }
}

impl TryFrom<&Literal> for ExprReturn {
    type Error = Error;

    fn try_from(literal: &Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::Nil => Ok(ExprReturn::Nil),
            Literal::Bool(v) => Ok(ExprReturn::Bool(*v)),
            Literal::Number(v) => Ok(ExprReturn::Number(*v)),
            Literal::String(v) => Ok(ExprReturn::String(v.to_owned())),
            // _ => Err(Error::LiteralError { literal }),
        }
    }
}

impl PartialEq for ExprReturn {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl Visitor<Result<ExprReturn>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<ExprReturn> {
        match expr {
            Expr::Literal(literal) => {
                let expr_return = ExprReturn::try_from(literal)?;
                Ok(expr_return)
            }
            Expr::Grouping { expr } => self.visit_expr(expr),
            Expr::Unary { operator, right } => {
                let right = self.visit_expr(right)?;

                match (operator.get_token_type(), right) {
                    (TokenType::Minus, ExprReturn::Number(v)) => Ok(ExprReturn::Number(-v)),
                    (TokenType::Bang, right) => Ok(ExprReturn::Bool(!right.is_truthy())),
                    (token_type, right) => Err(Error::UnsupportedUnary {
                        operator: token_type,
                        value: right,
                    }),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                match (left, operator.get_token_type(), right) {
                    (ExprReturn::Number(left), TokenType::Minus, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Number(left - right))
                    }
                    (ExprReturn::Number(left), TokenType::Slash, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Number(left / right))
                    }
                    (ExprReturn::Number(left), TokenType::Star, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Number(left * right))
                    }
                    (ExprReturn::Number(left), TokenType::Plus, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Number(left + right))
                    }
                    (ExprReturn::String(left), TokenType::Plus, ExprReturn::String(right)) => {
                        Ok(ExprReturn::String(format!("{}{}", left, right)))
                    }
                    (ExprReturn::Number(left), TokenType::Greater, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Bool(left > right))
                    }
                    (
                        ExprReturn::Number(left),
                        TokenType::GreaterEqual,
                        ExprReturn::Number(right),
                    ) => Ok(ExprReturn::Bool(left >= right)),
                    (ExprReturn::Number(left), TokenType::Less, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Bool(left < right))
                    }
                    (ExprReturn::Number(left), TokenType::LessEqual, ExprReturn::Number(right)) => {
                        Ok(ExprReturn::Bool(left <= right))
                    }
                    (left, TokenType::BangEqual, right) => Ok(ExprReturn::Bool(left != right)),
                    (left, TokenType::EqualEqual, right) => Ok(ExprReturn::Bool(left == right)),
                    (left, token_type, right) => Err(Error::UnsupportedBinary {
                        operator: token_type,
                        left,
                        right,
                    }),
                }
            }
        }
    }
}
