use std::fmt;

use thiserror::Error;

use super::{
    ast::{Expr, Literal, Visitor},
    token::Token,
    token_type::TokenType,
};

#[derive(Debug, Error)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Error converting literal {literal}.")]
    LiteralConversion { literal: Literal },

    #[error(
        "Operator `{operator}` expected one of: [{}], found {} of type {}.",
        .expected.join(", "),
        .found,
        .found.type_of()
    )]
    InvalidOperand {
        operator: TokenType,
        expected: Vec<String>,
        found: ExprReturn,
    },

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
            Literal::Bool(b) => Ok(ExprReturn::Bool(*b)),
            Literal::Number(n) => Ok(ExprReturn::Number(*n)),
            Literal::String(s) => Ok(ExprReturn::String(s.to_owned())),
            // _ => Err(Error::LiteralConversion { literal }),
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

impl fmt::Display for ExprReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => b.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::String(s) => write!(f, "{:?}", s),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&self, expr: &Expr) -> Result<ExprReturn> {
        self.visit_expr(expr)
    }
}

impl Visitor<Result<ExprReturn>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<ExprReturn> {
        match expr {
            Expr::Literal(literal) => {
                let expr_return = ExprReturn::try_from(literal)?;
                Ok(expr_return)
            }

            Expr::Grouping(expr) => self.visit_expr(expr),

            Expr::Unary { operator, right } => {
                let right = self.visit_expr(right)?;

                match operator.get_token_type() {
                    TokenType::Minus => match right {
                        ExprReturn::Number(n) => Ok(ExprReturn::Number(-n)),
                        _ => invalid_operand_error(operator, &["Number"], right),
                    },
                    TokenType::Bang => Ok(ExprReturn::Bool(!right.is_truthy())),
                    op => Err(Error::UnsupportedUnary {
                        operator: op,
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

                match operator.get_token_type() {
                    TokenType::Minus => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Number(l - r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Slash => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Number(l / r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Star => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Number(l * r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Plus => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Number(l + r))
                        }
                        (ExprReturn::String(l), ExprReturn::String(r)) => {
                            Ok(ExprReturn::String(format!("{}{}", l, r)))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (ExprReturn::String(_), right) => {
                            invalid_operand_error(operator, &["String"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number", "String"], left),
                    },
                    TokenType::Greater => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Bool(l > r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Bool(l >= r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Less => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Bool(l < r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (ExprReturn::Number(l), ExprReturn::Number(r)) => {
                            Ok(ExprReturn::Bool(l <= r))
                        }
                        (ExprReturn::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::BangEqual => Ok(ExprReturn::Bool(left != right)),
                    TokenType::EqualEqual => Ok(ExprReturn::Bool(left == right)),
                    _ => Err(Error::UnsupportedBinary {
                        operator: operator.get_token_type(),
                        left,
                        right,
                    }),
                }
            }
        }
    }
}

fn invalid_operand_error<V, S: ToString>(
    operator: &Token,
    expected: &[S],
    found: ExprReturn,
) -> Result<V> {
    Err(Error::InvalidOperand {
        operator: operator.get_token_type(),
        expected: expected.iter().map(ToString::to_string).collect(),
        found,
    })
}
