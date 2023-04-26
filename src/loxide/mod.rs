use std::io::Write;

use crate::loxide::scanner::Scanner;

mod scanner;
mod token;

pub struct Loxide {
    had_error: bool,
}

impl Loxide {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    fn run(&self, source: String) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        // For now, just print the tokens
        for token in tokens {
            println!("{:?}", token);
        }
    }

    pub fn run_file(&self, path: String) {
        let source = std::fs::read_to_string(path).expect("Failed to read file");
        self.run(source);
        if self.had_error {
            std::process::exit(65);
        }
    }

    pub fn run_repl(&mut self) {
        // Create a buffer to read input into
        let mut buffer = String::new();

        // Create a reader to read input from stdin
        let stdin = std::io::stdin();

        // Create a handle to stdout
        let mut stdout = std::io::stdout();

        loop {
            // Print the prompt
            print!("> ");
            stdout.flush().expect("Failed to flush stdout");

            // Read a line from stdin
            stdin
                .read_line(&mut buffer)
                .expect("Failed to read line from stdin");

            // If the buffer is empty, break
            if buffer.is_empty() {
                println!("Exiting...");
                break;
            }

            // Run the line
            self.run(buffer.clone());
            self.had_error = false;

            // Clear the buffer
            buffer.clear();

            // Flush stdout
            stdout.flush().expect("Failed to flush stdout");
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }
}
