use lexical::Keywords;
use std::{cell::RefCell, env, fs, rc::Rc};
mod lexical;
mod throw;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        panic!("Error: no arguments found.")
    }
    let source_path = &args[1];
    print!("source path: {}", source_path);
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("unable to read from source_path: {}", source_path));

    run_file(&source);
}

fn run_file(source: &str) {
    //init
    let errors = Rc::new(RefCell::new(throw::Error::new()));
    let mut scanner = lexical::Scanner::new(source, Rc::clone(&errors), Keywords::new());

    scanner.run();
    scanner.print();

    // Check for errors after running the scanner
    throw::check_errors(&errors.borrow());
}
