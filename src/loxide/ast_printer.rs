use super::{
    ast::{Expr, Visitor},
    token_type::TokenType,
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn visit_expr(&self, expr: &Expr) -> String {
        Visitor::visit_expr(self, expr)
    }
}

impl Visitor for AstPrinter {
    type ExprReturn = String;

    fn visit_expr(&self, expr: &Expr) -> Self::ExprReturn {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.get_lexeme(),
                self.visit_expr(left),
                self.visit_expr(right),
            ),
            Expr::Grouping { expr } => format!("(group {})", self.visit_expr(expr)),
            Expr::Literal { value } => match value.get_token_type() {
                TokenType::Nil => String::from("nil"),
                TokenType::True => String::from("true"),
                TokenType::False => String::from("false"),
                TokenType::Number(v) => v.to_string(),
                TokenType::String(v) => v,
                _ => panic!("Invalid token type"),
            },
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.get_lexeme(), self.visit_expr(right))
            }
        }
    }
}
