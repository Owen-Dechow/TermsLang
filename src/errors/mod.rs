use colored::*;
use std::fs;
use std::path::PathBuf;

#[macro_use]
mod macros;

#[derive(Debug)]
pub struct ManagerError(pub String, pub FileLocation);
from_for_err_macro! {ManagerError}
impl ManagerError {
    prettify_macro! {"Manager Error"}
}

#[derive(Debug)]
pub struct LexerError(pub String, pub FileLocation);
from_for_err_macro! {LexerError}
impl LexerError {
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

pub struct AParserError(pub String, pub FileLocation);
from_for_err_macro! {AParserError}
impl AParserError {
    prettify_macro! {"Active Parser Error"}
}

pub struct LspError(pub String, pub FileLocation);
from_for_err_macro! {LspError}
impl LspError {
    prettify_macro! {"Lsp Error"}
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileLocation {
    Loc {
        file: PathBuf,
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
    },
    End {
        file: PathBuf,
    },
    None,
}
impl FileLocation {
    pub fn start(&self) -> (usize, usize) {
        match self {
            FileLocation::Loc {
                start_line,
                start_col,
                ..
            } => (*start_line, *start_col),
            _ => panic!(),
        }
    }
}

pub enum ErrorType {
    Lsp(LspError),
    Parser(ParserError),
    AParser(AParserError),
    Lexer(LexerError),
}
