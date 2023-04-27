use super::ast::{Expr, Literal, Visitor};

pub struct AstPrinter;

impl AstPrinter {
    #[allow(dead_code)]
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&self, expr: &Expr) -> String {
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
            Expr::Grouping(expr) => format!("(group {})", self.visit_expr(expr)),
            Expr::Literal(literal) => match literal {
                Literal::Nil => String::from("nil"),
                Literal::Bool(v) => v.to_string(),
                Literal::Number(v) => v.to_string(),
                Literal::String(v) => v.to_owned(),
            },
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.get_lexeme(), self.visit_expr(right))
            }
        }
    }
}
