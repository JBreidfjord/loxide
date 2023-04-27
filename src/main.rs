use loxide::{Error, Loxide};

mod loxide;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut loxide = Loxide::new();
    match args.len() {
        1 => loxide.run_repl().unwrap(),
        2 => {
            if let Err(e) = loxide.run_file(&args[1]) {
                println!("{}", e);
                std::process::exit(match e {
                    Error::Runtime(_) => 70,
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
