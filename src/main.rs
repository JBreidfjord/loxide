use loxide::Loxide;

mod loxide;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut loxide = Loxide::new();
    match args.len() {
        1 => loxide.run_repl(),
        2 => loxide.run_file(args[0].clone()),
        _ => {
            println!("Usage: loxide [script]");
            std::process::exit(64);
        }
    }
}
