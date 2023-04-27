use std::fmt;

use super::token::Token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Literal {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

pub trait Visitor<R> {
    fn visit_expr(&self, expr: &Expr) -> R;
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Bool(v) => write!(f, "{}", v),
            Literal::Number(v) => write!(f, "{}", v),
            Literal::String(v) => write!(f, "{}", v),
        }
    }
}
