#[macro_use]
mod macros;

#[derive(Debug)]
pub struct LexerError(pub String, pub Option<FileLocation>);
impl LexerError {
    prettify_macro! {"Lexer Error"}
}

#[derive(Debug)]
pub struct TranspilerError(pub String, pub Option<FileLocation>);
impl TranspilerError {
    prettify_macro! {"Lexer Error"}
}

#[derive(Debug)]
pub struct ParserError(pub String, pub Option<FileLocation>);
impl ParserError {
    prettify_macro! {"Parser Error"}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileLocation {
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
}
