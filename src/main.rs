mod ast;
mod diagnostics;
mod lexer;
mod parser;
mod runtime;

use std::{env, fs, process};

use runtime::interpreter::Interpreter;

fn usage() {
    println!("KODE programming language v0.2.0");
    println!();
    println!("Commands:");
    println!("  kode run <file.kode>");
    println!("  kode check <file.kode>");
    println!("  kode --version");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "--version" {
        println!("KODE 0.2.0");
        return;
    }

    if args.len() != 3 {
        usage();
        process::exit(2);
    }

    let command = &args[1];
    let filename = &args[2];

    let source = match fs::read_to_string(filename) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("KODE error: cannot read '{}': {}", filename, error);
            process::exit(1);
        }
    };

    let tokens = match lexer::lex(&source) {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };

    let program = match parser::parse(tokens) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };

    match command.as_str() {
        "check" => {
            println!("KODE: syntax is valid.");
        }

        "run" => {
            let mut interpreter = Interpreter::new();

            if let Err(error) = interpreter.execute(&program) {
                eprintln!("{}", error);
                process::exit(1);
            }
        }

        _ => {
            usage();
            process::exit(2);
        }
    }
}