use super::token::Token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

pub trait Visitor {
    type ExprReturn;

    fn visit_expr(&self, expr: &Expr) -> Self::ExprReturn;
}
