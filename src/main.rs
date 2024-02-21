pub mod errors;
pub mod lexer;
pub mod parser;
mod terms2rust_transpiler;

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
        Ok(block) => match terms2rust_transpiler::transpile(block) {
            Ok(out) => {
                let _ = fs::write("output/src/main.rs", out);
            }
            Err(err) => panic!("{}", err.prettify(&program)),
        },
        Err(err) => {
            println!("{}", err.prettify(&program));
            panic!("{}", err.prettify(&program));
        }
    }
}
