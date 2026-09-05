mod token;
mod lexer;
mod ast;
mod parser;
mod value;
mod error;
mod packages;
mod networking;
mod gui;
mod interpreter;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::{Interpreter, StdinInputProvider};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_repl();
    } else if args.len() == 3 && args[1] == "-e" {
        run_inline(&args[2]);
    } else if args.len() == 2 {
        if args[1] == "--version" || args[1] == "-v" {
            println!("Y++ 1.2.0 (Native Release with yGUI)");
        } else if args[1] == "--help" || args[1] == "-h" {
            print_help();
        } else {
            run_file(&args[1]);
        }
    } else {
        print_help();
        process::exit(1);
    }
}

fn print_help() {
    println!("Y++ Programming Language Interpreter v1.2.0");
    println!("Usage:");
    println!("  ypp               (Starts interactive REPL)");
    println!("  ypp <file.ypp>    (Executes a Y++ source file)");
    println!("  ypp -e \"<code>\"   (Executes inline Y++ code)");
    println!("  ypp -v, --version (Prints version)");
    println!("  ypp -h, --help    (Prints this help message)");
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", path, e);
            process::exit(1);
        }
    };
    if let Err(e) = execute(&source, true) {
        eprintln!("[Y++ Error] {}", e);
        process::exit(1);
    }
}

fn run_inline(code: &str) {
    if let Err(e) = execute(code, true) {
        eprintln!("[Y++ Error] {}", e);
        process::exit(1);
    }
}

fn run_repl() {
    println!("Y++ 1.2.0 (Rust Interactive Shell with yGUI)");
    println!("Type 'help', 'clear', or 'exit' for options.");
    
    let mut interpreter = Interpreter::new(Box::new(StdinInputProvider));
    
    // Auto-inject Import ycomponents * for REPL convenience
    let import_stmt = "Import ycomponents *;";
    let mut lexer = Lexer::new(import_stmt);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    if let Ok(ast) = parser.parse() {
        let _ = interpreter.execute_program(&ast);
    }
    
    let mut buffer = String::new();
    let mut depth = 0;

    loop {
        let prompt = if depth == 0 { "ypp> " } else { "...  " };
        print!("{}", prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if depth == 0 {
                    if trimmed == "exit" || trimmed == "quit" {
                        break;
                    }
                    if trimmed == "clear" {
                        interpreter = Interpreter::new(Box::new(StdinInputProvider));
                        let mut lexer = Lexer::new(import_stmt);
                        if let Ok(ast) = Parser::new(lexer.tokenize()).parse() {
                            let _ = interpreter.execute_program(&ast);
                        }
                        println!("Environment cleared.");
                        buffer.clear();
                        continue;
                    }
                    if trimmed == "help" {
                        println!("Y++ REPL Help:");
                        println!("- Type Y++ statements and press Enter to execute.");
                        println!("- Use 'clear' to reset environment, 'exit' to quit.");
                        continue;
                    }
                }

                for c in line.chars() {
                    if c == '{' { depth += 1; }
                    if c == '}' && depth > 0 { depth -= 1; }
                }

                buffer.push_str(&line);

                if depth == 0 && !buffer.trim().is_empty() {
                    let mut lexer = Lexer::new(&buffer);
                    let tokens = lexer.tokenize();
                    let mut parser = Parser::new(tokens);
                    match parser.parse() {
                        Ok(ast) => {
                            if let Err(e) = interpreter.execute_program(&ast) {
                                eprintln!("[Y++ Error] {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Y++ Parse Error] {}", e);
                        }
                    }
                    buffer.clear();
                }
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn execute(source: &str, require_import: bool) -> Result<(), crate::error::YppError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    let mut interpreter = Interpreter::new(Box::new(StdinInputProvider));
    if !require_import {
        let import_stmt = "Import ycomponents *;";
        let mut l = Lexer::new(import_stmt);
        let t = l.tokenize();
        let mut p = Parser::new(t);
        if let Ok(a) = p.parse() {
            let _ = interpreter.execute_program(&a);
        }
    }
    interpreter.execute_program(&ast)
}
