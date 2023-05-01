use std::fmt;

use super::{interpreter::functions::Function, token::Token};

#[derive(Clone)]
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
    Variable(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Break,
    Function(Function),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

pub trait Visitor<E, S> {
    fn visit_expr(&mut self, expr: &Expr) -> E;
    fn visit_stmt(&mut self, stmt: &Stmt) -> S;
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
