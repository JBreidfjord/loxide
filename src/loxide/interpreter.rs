use std::fmt;

use thiserror::Error;

use super::{
    ast::{Expr, Literal, Stmt, Visitor},
    token::Token,
    token_type::TokenType,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Operator `{operator}` expected one of: [{}], found {} of type {}.",
        .expected.join(", "),
        .found,
        .found.type_of()
    )]
    InvalidOperand {
        operator: TokenType,
        expected: Vec<String>,
        found: Value,
    },

    #[error("Unsupported unary operator `{operator}` on type {}.", .value.type_of())]
    UnsupportedUnary { operator: TokenType, value: Value },

    #[error(
        "Unsupported binary operator `{operator}` on types {} and {}",
        .left.type_of(),
        .right.type_of()
    )]
    UnsupportedBinary {
        operator: TokenType,
        left: Value,
        right: Value,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

impl Value {
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

impl TryFrom<&Literal> for Value {
    type Error = Error;

    fn try_from(literal: &Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::Nil => Ok(Value::Nil),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Number(n) => Ok(Value::Number(*n)),
            Literal::String(s) => Ok(Value::String(s.to_owned())),
        }
    }
}

impl PartialEq for Value {
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => b.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::String(s) => write!(f, "{:?}", s),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }
}

impl Visitor<Result<Value>, Result<()>> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.visit_expr(expr)?;
            }
            Stmt::Print(expr) => println!("{}", self.visit_expr(expr)?),
            Stmt::Var { name, initializer } => todo!(),
        }

        Ok(())
    }

    fn visit_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(literal) => {
                let value = Value::try_from(literal)?;
                Ok(value)
            }

            Expr::Grouping(expr) => self.visit_expr(expr),

            Expr::Unary { operator, right } => {
                let right = self.visit_expr(right)?;

                match operator.get_token_type() {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => invalid_operand_error(operator, &["Number"], right),
                    },
                    TokenType::Bang => Ok(Value::Bool(!right.is_truthy())),
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
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => {
                            Ok(Value::String(format!("{}{}", l, r)))
                        }
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (Value::String(_), right) => {
                            invalid_operand_error(operator, &["String"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number", "String"], left),
                    },
                    TokenType::Greater => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Less => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    _ => Err(Error::UnsupportedBinary {
                        operator: operator.get_token_type(),
                        left,
                        right,
                    }),
                }
            }

            Expr::Variable(_) => todo!(),
        }
    }
}

fn invalid_operand_error<V, S: ToString>(
    operator: &Token,
    expected: &[S],
    found: Value,
) -> Result<V> {
    Err(Error::InvalidOperand {
        operator: operator.get_token_type(),
        expected: expected.iter().map(ToString::to_string).collect(),
        found,
    })
}
