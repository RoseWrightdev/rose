use std::env;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

mod lexical;
mod parser;
mod throw;
mod abstract_syntax_tree;

use lexical::scanner::Scanner;
use lexical::keywords::Keywords; 
use throw::Error as ReporterError;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file_path>", args.get(0).map_or("program", |s| s.as_str()));
        std::process::exit(1);
    }
    let source_path = &args[1];
    println!("Source path: {}", source_path);

    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading from source_path '{}': {}", source_path, e);
            std::process::exit(1);
        }
    };

    run_file(&source);
}

fn run_file(source: &str) {
    // --- Scanner Phase ---
    println!("\n--- Running Scanner ---");
    let error_reporter = Rc::new(RefCell::new(ReporterError::new())); 
    let keywords = Keywords::new(); 
    let mut scanner = Scanner::new(source, Rc::clone(&error_reporter), keywords);
    
    // Scanner's `run` method now directly returns &Vec<Token>
    let tokens = scanner.run(); 

    // Optionally print tokens for debugging
    // scanner.print_tokens();

    // Check for scanner errors
    if error_reporter.borrow().has_error_occurred() { 
        println!("Lexical errors found:");
        // Using the Display trait implemented on ReporterError:
        eprintln!("{}", error_reporter.borrow().to_string()); 
        return; // Stop if there are scanning errors
    }
    println!("Scanner finished successfully. {} tokens generated.", tokens.len());

    // --- Parser Phase ---
    println!("\n--- Running Parser ---");
    // The parser takes ownership of the tokens, so we clone them.
    let mut parser = parser::Parser::new(tokens.clone()); 
    
    match parser.parse_program() {
        Ok(program_ast) => {
            println!("Parser finished successfully!");
            println!("\n--- Abstract Syntax Tree (AST) ---");
            // Pretty print the AST (or a summary)
            // Using {:#?} for a more detailed debug print.
            for (i, stmt) in program_ast.iter().enumerate() {
                println!("Statement {}: {:#?}", i + 1, stmt);
            }
            if program_ast.is_empty() && !tokens.iter().any(|t| t.token_type != lexical::TokenType::EndOfFile) {
                println!("(Program is empty or contains only EOF tokens)");
            } else if program_ast.is_empty() {
                 println!("(Program is empty, but tokens were present. Check parser logic for top-level statements.)");
            }

        }
        Err(parse_errors) => {
            println!("\n--- Parser Errors ---");
            eprintln!("Found {} parsing error(s):", parse_errors.len());
            for error in parse_errors {
                let token_info = error.token.as_ref().map_or_else(
                    || "at end of input".to_string(),
                    |t| format!("near token '{}' (type: {:?})", t.lexeme, t.token_type)
                );
                eprintln!("[Line {}] Error {}: {}", error.line, token_info, error.message);
            }
        }
    }
}
