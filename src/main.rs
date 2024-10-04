mod active_parser;
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
        return;
    }

    // Lex file
    let lex_out = match lexer::lex(&program, false, &args.file) {
        Ok(lex) => lex,
        Err(err) => {
            println!("{}", err.prettify());
            return;
        }
    };

    // Parse lex
    let parse_out = match parser::parse(lex_out, &args.file) {
        Ok(parse) => parse,
        Err(err) => {
            match err {
                parser::ErrType::Parser(err) => println!("{}", err.prettify()),
                parser::ErrType::Lexer(err) => println!("{}", err.prettify()),
            };
            return;
        }
    };

    let aparse = match active_parser::aparse(&parse_out) {
        Ok(out) => out,
        Err(err) => {
            println!("{}", err.prettify());
            return;
        }
    };
    println!("{:?}\n", aparse);

    // Run program
    let _interpretor_out = match interpretor::interpret(parse_out) {
        Ok(intperpretation) => intperpretation,
        Err(err) => {
            println!("{}", err.prettify());
            return;
        }
    };
}
