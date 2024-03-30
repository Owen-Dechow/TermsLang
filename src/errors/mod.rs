#[macro_use]
mod macros;

#[derive(Debug)]
pub struct LexerError(pub String, pub FileLocation);
from_for_err_macro! {LexerError}
impl LexerError {
    prettify_macro! {"Lexer Error"}
}

#[derive(Debug)]
pub struct TranspilerError(pub String, pub FileLocation);
from_for_err_macro! {TranspilerError}
impl TranspilerError {
    prettify_macro! {"Lexer Error"}
}

#[derive(Debug)]
pub struct ParserError(pub String, pub FileLocation);
from_for_err_macro! {ParserError}
impl ParserError {
    prettify_macro! {"Parser Error"}
}

pub struct RuntimeError(pub String, pub FileLocation);
from_for_err_macro! {RuntimeError}
impl RuntimeError {
    prettify_macro! {"Runtime Error"}
}

pub struct ActiveParserError(pub String, pub FileLocation);
from_for_err_macro! {ActiveParserError}
impl ActiveParserError {
    prettify_macro! {"Active Parser Error"}
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileLocation {
    Loc {
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
    },
    End,
    None,
}
