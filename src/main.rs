pub mod errors;
pub mod interpretor;
pub mod lexer;
pub mod llvm_compiler;
pub mod parser;

use std::fs;

fn main() {
    // Read input file
    let program = {
        let mut program = fs::read_to_string("in.txt").expect("Can not read input file");
        program.push(' ');
        program
    };

    // Lex file
    let lex_out = match lexer::lex(&program) {
        Ok(lex) => lex,
        Err(err) => {
            panic!("{}", err.prettify(&program));
        }
    };

    // Parse lex
    let parse_out = match parser::parse(lex_out) {
        Ok(parse) => parse,
        Err(err) => {
            panic!("{}", err.prettify(&program));
        }
    };

    // Run prog
    let _interpretor_out = match interpretor::interpret(parse_out) {
        Ok(intperpretation) => intperpretation,
        Err(err) => {
            panic!("{}", err.prettify(&program));
        }
    };
}
