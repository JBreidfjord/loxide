use loxide::ast::Expr;
use loxide::ast_printer::AstPrinter;
use loxide::token::Token;
use loxide::token_type::TokenType;
use loxide::{Error, Loxide};

mod loxide;

fn main() {
    // Print test AST
    let expression = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token::new(TokenType::Minus, String::from("-"), 1),
            right: Box::new(Expr::Literal {
                value: Token::new(TokenType::Number(123.0), String::from("123"), 1),
            }),
        }),
        operator: Token::new(TokenType::Star, String::from("*"), 1),
        right: Box::new(Expr::Grouping {
            expr: Box::new(Expr::Literal {
                value: Token::new(TokenType::Number(45.67), String::from("45.67"), 1),
            }),
        }),
    };
    println!("{}", AstPrinter.visit_expr(&expression));

    let args = std::env::args().collect::<Vec<String>>();
    let mut loxide = Loxide::new();
    match args.len() {
        1 => loxide.run_repl().unwrap(),
        2 => {
            if let Err(e) = loxide.run_file(&args[0]) {
                println!("{}", e);
                std::process::exit(match e {
                    Error::Io(_) => 74,
                    _ => 65,
                });
            }
        }
        _ => {
            println!("Usage: loxide [script]");
            std::process::exit(64);
        }
    }
}
