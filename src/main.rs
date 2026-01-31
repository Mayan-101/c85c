// src/main.rs

use std::env;
use std::fs;
use std::path::Path;

// Declare all our modules
mod lexer;
mod parser;
mod codegen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: c85c <input_file.c85>");
        std::process::exit(1);
    }
    let input_path = &args[1];

    let source_code = fs::read_to_string(input_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", input_path, err);
        std::process::exit(1)
    });

    // 1. Lex the source code into tokens.
    let tokens = lexer::lex(&source_code).unwrap_or_else(|err| {
        eprintln!("Lexer Error: {}", err);
        std::process::exit(1)
    });

    // 2. Parse the tokens into an AST.
    let ast = parser::parse(&tokens).unwrap_or_else(|err| {
        eprintln!("Parsing Error: {}", err);
        std::process::exit(1)
    });

    // 3. Generate the assembly code from the AST.
    let asm_code = codegen::generate(&ast);

    // 4. Write the output to an .asm file.
    let output_path = Path::new(input_path).with_extension("asm");
    fs::write(&output_path, asm_code).unwrap_or_else(|err| {
        eprintln!("Error writing to file '{}': {}", output_path.to_str().unwrap(), err);
        std::process::exit(1)
    });

    println!("✅ Compilation successful! Output written to {}", output_path.to_str().unwrap());
}