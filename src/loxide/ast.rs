use std::fmt;

use ordered_float::OrderedFloat;

use super::{interpreter::functions::FunctionDeclaration, token::Token};

#[derive(Clone, PartialEq, Eq, Hash)]
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
    Lambda(FunctionDeclaration),
    Get {
        object: Box<Expr>,
        name: Token,
    },
}

#[derive(Clone, PartialEq, Eq, Hash)]
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
    Function(FunctionDeclaration),
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<FunctionDeclaration>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Nil,
    Number(OrderedFloat<f64>),
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
