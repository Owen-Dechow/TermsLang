#[macro_use]
mod macros;

#[derive(Debug)]
pub struct LexerError(pub String, pub FileLocation);
impl LexerError {
    prettify_macro! {"Lexer Error"}
}

#[derive(Debug)]
pub struct TranspilerError(pub String, pub FileLocation);
impl TranspilerError {
    prettify_macro! {"Lexer Error"}
}
from_for_err_macro! {TranspilerError}

#[derive(Debug)]
pub struct ParserError(pub String, pub FileLocation);
impl ParserError {
    prettify_macro! {"Parser Error"}
}

pub struct RuntimeError(pub String, pub FileLocation);
impl RuntimeError {
    prettify_macro! {"Runtime Error"}
}

pub struct ParserCheckerError(pub String, pub FileLocation);

#[derive(Debug, PartialEq, Clone)]
pub enum FileLocation {
    Loc {
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
    },
    EOF,
}
