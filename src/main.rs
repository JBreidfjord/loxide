use std::io::Write;

fn run(source: String) {
    todo!();
}

fn run_file(path: String) {
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    run(source);
}

fn run_repl() {
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
        run(buffer.clone());

        // Clear the buffer
        buffer.clear();

        // Flush stdout
        stdout.flush().expect("Failed to flush stdout");
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    match args.len() {
        1 => run_repl(),
        2 => run_file(args[0].clone()),
        _ => {
            println!("Usage: loxide [script]");
            std::process::exit(64);
        }
    }
}
