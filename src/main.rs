mod cli;
pub mod errors;
mod formmatter;
pub mod interpretor;
pub mod lexer;
pub mod parser;

use clap::Parser;
use std::fs;

fn main() {
    let args = cli::Args::parse();
    // Read input file
    let program = {
        let mut program = fs::read_to_string(&args.file).expect("Can not read input file");
        program.push(' ');
        program
    };

    if args.format {
        // formmat program text
        let text = formmatter::format(&program, 4);
        fs::write(&args.file, text).expect("Cannot write to input file");
    } else {
        // Lex file
        let lex_out = match lexer::lex(&program, false) {
            Ok(lex) => lex,
            Err(err) => {
                println!("{}", err.prettify(&program));
                return;
            }
        };

        // Parse lex
        let parse_out = match parser::parse(lex_out) {
            Ok(parse) => parse,
            Err(err) => {
                println!("{}", err.prettify(&program));
                return;
            }
        };

        // Run program
        let _interpretor_out = match interpretor::interpret(parse_out) {
            Ok(intperpretation) => intperpretation,
            Err(err) => {
                println!("{}", err.prettify(&program));
                return;
            }
        };
    }
}
