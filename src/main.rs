pub mod errors;
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
            println!("{}", err.prettify(&program));
            panic!("{}", err.prettify(&program));
        }
    };

    // Parse lex
    let parse_out = parser::parse(lex_out);

    match &parse_out {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err.prettify(&program));
            panic!("{}", err.prettify(&program));
        }
    }
}
