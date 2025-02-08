use crate::{
    active_parser::{aparse, names},
    errors::{AParserError, FileLocation, LexerError, LspError, ParserError},
    lexer::tokens::{KeyWord, Operator, Token, TokenType},
    parser::{parse, ErrType},
};
use std::{collections::HashMap, path::PathBuf};

struct TS(Vec<Token>, usize);
impl TS {
    fn new(tokens: Vec<Token>) -> TS {
        TS(tokens, 0)
    }

    fn next(&mut self) -> Option<&Token> {
        let t = self.0.get(self.1);
        self.1 += 1;
        return t;
    }

    fn back(&mut self) {
        self.1 -= 1;
    }
}

pub struct Lsp {
    vars: Vec<HashMap<String, ((usize, usize), String)>>,
    errors: Vec<ErrorType>,
}

impl Lsp {
    fn insert(
        &mut self,
        k: String,
        v: ((usize, usize), String),
        loc: &FileLocation,
    ) -> Result<(), LspError> {
        match self.vars.last_mut() {
            Some(some) => some,
            None => {
                return Err(LspError(
                    format!("Found variable outside of scope."),
                    loc.clone(),
                ))
            }
        }
        .insert(k, v);
        return Ok(());
    }

    pub fn json(self) -> String {
        let mut string = String::from("{\"variables\":{");

        for scope in self.vars {
            for var in scope {
                string += &format!("\"{}\":{{", var.0);
                string += &format!("\"line\":{},", var.1 .0 .0);
                string += &format!("\"col\":{},", var.1 .0 .1);
                string += &format!("\"type\":\"{}\"}},", var.1 .1);
            }
        }

        string = match string.strip_suffix(',') {
            Some(s) => s.to_string(),
            None => string,
        };

        string += "},\"errors\":[";

        for error in self.errors {
            let json = match error {
                ErrorType::Lsp(lsp_error) => lsp_error.json(),
                ErrorType::Parser(parser_error) => parser_error.json(),
                ErrorType::AParser(aparser_error) => aparser_error.json(),
                ErrorType::Lexer(lexer_error) => lexer_error.json(),
            };
            string += &json;
            string += ",";
        }

        string = match string.strip_suffix(',') {
            Some(s) => s.to_string(),
            None => string,
        };

        string += "]}";

        return string;
    }
}

fn get_type(ts: &mut TS) -> Option<String> {
    let mut list = 0;
    if let Some(Token(TokenType::Identity(arg_type), _)) = ts.next() {
        let var_type = arg_type.clone();
        while let Some(Token(TokenType::Operator(Operator::OpenBracket), _)) = ts.next() {
            if let Some(Token(TokenType::Operator(Operator::CloseBracket), _)) = ts.next() {
                list += 1;
                continue;
            }

            return None;
        }

        ts.back();
        return format!("{}{}", var_type, String::from("[]").repeat(list)).into();
    }

    return None;
}

fn get_args(ts: &mut TS, lsp: &mut Lsp) -> Result<(), LspError> {
    if let Some(arg_type) = get_type(ts) {
        let arg_type = arg_type.clone();
        if let Some(Token(TokenType::Identity(id), loc)) = ts.next() {
            lsp.insert(id.clone(), (loc.start(), arg_type), loc)?;

            if let Some(Token(TokenType::Operator(Operator::Comma), _)) = ts.next() {
                get_args(ts, lsp)?;
            } else {
                ts.back();
            }
        }
    }

    return Ok(());
}

pub enum ErrorType {
    Lsp(LspError),
    Parser(ParserError),
    AParser(AParserError),
    Lexer(LexerError),
}

fn get_vars(ts: &mut TS, line: usize, col: usize) -> Result<Lsp, LspError> {
    let mut lsp = Lsp {
        vars: Vec::new(),
        errors: Vec::new(),
    };

    while let Some(t) = ts.next() {
        let (token, loc) = (&t.0, &t.1);

        let start = loc.start();
        if start.0 > line {
            break;
        } else if start.0 == line {
            if start.1 > col {
                break;
            }
        }

        if let TokenType::KeyWord(KeyWord::Var) = token {
            if let Some(var_type) = get_type(ts) {
                let var_type = var_type.clone();

                if let Some(Token(TokenType::Identity(id), loc)) = ts.next() {
                    lsp.insert(id.clone(), (loc.start(), var_type), loc)?;
                }
            }
        } else if let TokenType::KeyWord(KeyWord::Func) = token {
            lsp.vars.push(HashMap::new());

            if let Some(_) = get_type(ts) {
                if let Some(Token(TokenType::Identity(_), _)) = ts.next() {
                    if let Some(Token(TokenType::Operator(Operator::Colon), _)) = ts.next() {
                        get_args(ts, &mut lsp)?;
                    }
                }
            }
        } else if let TokenType::Operator(Operator::CloseBlock) = token {
            lsp.vars.pop();
        } else if let TokenType::KeyWord(KeyWord::If) = token {
            lsp.vars.push(HashMap::new());

            if let Some(Token(TokenType::KeyWord(KeyWord::Else), _)) = ts.next() {
            } else {
                ts.back();
            }
        } else if let TokenType::KeyWord(KeyWord::Else) = token {
            lsp.vars.push(HashMap::new());
        } else if let TokenType::KeyWord(KeyWord::Loop) = token {
            lsp.vars.push(HashMap::new());
            if let Some(Token(TokenType::Identity(id), loc)) = ts.next() {
                lsp.insert(id.clone(), (loc.start(), names::INT.to_string()), loc)?;
            }
        } else if let TokenType::KeyWord(KeyWord::Struct) = token {
            lsp.vars.push(HashMap::new());
        }
    }

    return Ok(lsp);
}

pub fn lsp(prog: Vec<Token>, file: &PathBuf, line: usize, col: usize, run_parse: bool) -> Lsp {
    let mut errors = Vec::new();

    if run_parse {
        match parse(prog.clone(), file) {
            Ok(program) => match aparse(&program) {
                Ok(_) => {}
                Err(err) => errors.push(ErrorType::AParser(err)),
            },
            Err(err) => match err {
                ErrType::Parser(parser_error) => errors.push(ErrorType::Parser(parser_error)),
                ErrType::Lexer(lexer_error) => errors.push(ErrorType::Lexer(lexer_error)),
            },
        }
    }

    let mut ts = TS::new(prog);
    let lsp = match get_vars(&mut ts, line, col) {
        Ok(mut ok) => {
            ok.errors.append(&mut errors);
            ok
        }
        Err(err) => {
            errors.push(ErrorType::Lsp(err));
            Lsp {
                vars: Vec::new(),
                errors,
            }
        }
    };

    return lsp;
}
