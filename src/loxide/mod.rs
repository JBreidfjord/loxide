use std::io::Write;

use thiserror::Error;

use self::{parser::Parser, scanner::Scanner};

pub mod ast;
pub mod ast_printer;
mod parser;
mod scanner;
pub mod token;
pub mod token_type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}Scanning failed, see errors above.", .0.iter().map(|e| format!("{}\n", e)).collect::<String>())]
    Scanner(Vec<self::scanner::Error>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

pub struct Loxide;

impl Loxide {
    pub fn new() -> Self {
        Self
    }

    fn run(&self, source: Vec<u8>) -> Result {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().map_err(Error::Scanner)?;

        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        println!("{}", ast_printer::AstPrinter.visit_expr(&expr));

        Ok(())
    }

    pub fn run_file(&self, path: &str) -> Result {
        let source = std::fs::read(path)?;
        self.run(source)
    }

    pub fn run_repl(&mut self) -> Result {
        // Create a reader to read input from stdin
        let stdin = std::io::stdin();

        // Create a handle to stdout
        let mut stdout = std::io::stdout();

        loop {
            // Print the prompt
            print!("> ");
            stdout.flush()?;

            // Read a line from stdin
            let mut buffer = String::new();
            stdin.read_line(&mut buffer)?;

            // If the buffer is empty, break
            if buffer.is_empty() {
                println!("Exiting...");
                break;
            }

            // Run the line
            match self.run(buffer.into_bytes()) {
                // TODO: Print returned value if any
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            // Flush stdout
            stdout.flush()?;
        }

        Ok(())
    }
}
