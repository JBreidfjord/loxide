use std::io::Write;

use thiserror::Error;

use self::{interpreter::Interpreter, parser::Parser, resolver::Resolver, scanner::Scanner};

mod ast;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod token;
mod token_type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}Scanning failed, see errors above.", .0.iter().map(|e| format!("{e}\n")).collect::<String>())]
    Scanner(Vec<self::scanner::Error>),

    #[error("{}Parsing failed, see errors above.", .0.iter().map(|e| format!("{e}\n")).collect::<String>())]
    Parser(Vec<self::parser::Error>),

    #[error("{}Variable resolution failed, see errors above.", .0.iter().map(|e| format!("{e}\n")).collect::<String>())]
    Resolver(Vec<self::resolver::Error>),

    #[error(transparent)]
    Runtime(#[from] self::interpreter::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

pub struct Loxide {
    interpreter: Interpreter,
}

impl Loxide {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    fn run(&mut self, source: Vec<u8>) -> Result {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().map_err(Error::Scanner)?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse().map_err(Error::Parser)?;

        let locals = Resolver::new().run(&statements).map_err(Error::Resolver)?;
        self.interpreter.update_locals(locals);

        self.interpreter
            .interpret(&statements)
            .map_err(Error::Runtime)
    }

    pub fn run_file(&mut self, path: &str) -> Result {
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
                Ok(_) => {}
                Err(e) => println!("{e}"),
            }

            // Flush stdout
            stdout.flush()?;
        }

        Ok(())
    }
}
