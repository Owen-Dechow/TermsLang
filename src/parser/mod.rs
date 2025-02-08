use std::{fs, path::PathBuf};

use crate::{
    errors::{ErrorType, FileLocation, ParserError},
    lexer::{
        self,
        tokens::{KeyWord, Operator, Token, TokenType},
    },
};

use self::{
    parse_object::parse_object_peekable,
    parse_operand_block::{parse_operand_block, OperandExpression},
    parse_type::{parse_type, parse_var_sig},
};

pub mod parse_object;
pub mod parse_operand_block;
pub mod parse_type;

#[derive(Debug)]
pub struct TokenStream {
    ptr: usize,
    tokens: Vec<Token>,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> TokenStream {
        return TokenStream { ptr: 0, tokens };
    }

    fn current(&self) -> Option<&Token> {
        return self.tokens.get(self.ptr);
    }

    fn advance(&mut self) -> Option<&Token> {
        self.ptr += 1;
        return self.tokens.get(self.ptr);
    }

    fn back(&mut self) {
        self.ptr -= 1;
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Array {
        _type: Box<Type>,
        location: FileLocation,
    },
    Object {
        object: Object,
    },
}
impl Type {
    pub fn get_location(&self) -> &FileLocation {
        match self {
            Type::Array { location, .. } => location,
            Type::Object { object } => &object.loc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub loc: FileLocation,
    pub kind: ObjectType,
    pub sub: Option<Box<Object>>,
}

#[derive(Debug, Clone)]
pub struct ObjectCreate {
    pub kind: Type,
    pub args: Call,
}

#[derive(Debug, Clone)]
pub enum ObjectType {
    Identity(String),
    Call(Call),
    Index(Box<OperandExpression>),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub args: Vec<OperandExpression>,
}

#[derive(Debug, Clone)]
pub struct VarSigniture {
    pub identity: String,
    pub argtype: Type,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub returntype: Type,
    pub args: Vec<VarSigniture>,
    pub block: TermBlock,
    pub loc: FileLocation,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub properties: Vec<VarSigniture>,
    pub methods: Vec<Function>,
    pub loc: FileLocation,
}

#[derive(Debug, Clone)]
pub struct TermBlock {
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone)]
pub enum Term {
    Print {
        ln: bool,
        operand_block: OperandExpression,
    },
    DeclareVar {
        name: String,
        vartype: Type,
        value: OperandExpression,
    },
    Return {
        value: OperandExpression,
    },
    UpdateVar {
        var: Object,
        set_operator: Operator,
        value: OperandExpression,
    },
    If {
        conditional: OperandExpression,
        block: TermBlock,
        else_block: TermBlock,
    },
    Loop {
        counter: String,
        conditional: OperandExpression,
        block: TermBlock,
    },
    Break(FileLocation),
    Continue(FileLocation),
    Call {
        value: OperandExpression,
    },
}

// Parse single term
fn parse_term(
    lead_token: Token,
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<Term, ParserError> {
    // Parse print
    if let Token(TokenType::KeyWord(KeyWord::Print), _) = lead_token {
        return Ok(Term::Print {
            ln: false,
            operand_block: parse_operand_block(token_stream, vec![TokenType::Terminate], file)?,
        });
    }

    // Parse println
    if let Token(TokenType::KeyWord(KeyWord::PrintLn), _) = lead_token {
        return Ok(Term::Print {
            ln: true,
            operand_block: parse_operand_block(token_stream, vec![TokenType::Terminate], file)?,
        });
    }

    // Parse var declaration
    if let Token(TokenType::KeyWord(KeyWord::Var), _) = lead_token {
        let vartype = parse_type(token_stream, file)?;
        let name = match token_stream.advance().cloned() {
            Some(op) => match op.0 {
                TokenType::Identity(id) => id,
                _ => {
                    return Err(ParserError(
                        "Unexpected token in place of variable name".to_string(),
                        op.1,
                    ))
                }
            },
            None => {
                return Err(ParserError(
                    "Expected type variable name".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        };

        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Set), _)) => {}
            Some(Token(_, pos)) => {
                return Err(ParserError(
                    "Invalid token expected set operator".to_string(),
                    pos.clone(),
                ))
            }
            None => {
                return Err(ParserError(
                    "Expected set operator".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        }

        let value = parse_operand_block(token_stream, vec![TokenType::Terminate], file)?;

        return Ok(Term::DeclareVar {
            name: name.to_owned(),
            vartype,
            value,
        });
    }

    // Parse return
    if let Token(TokenType::KeyWord(KeyWord::Return), _) = lead_token {
        return Ok(Term::Return {
            value: parse_operand_block(token_stream, vec![TokenType::Terminate], file)?,
        });
    }

    // Parse var update
    if let Token(TokenType::KeyWord(KeyWord::UpdateVar), _) = lead_token {
        let var = parse_object_peekable(token_stream, file)?;

        let set_operator = match token_stream.advance() {
            Some(Token(TokenType::Operator(operator), pos)) => match operator {
                Operator::Set => operator,
                Operator::SetAdd => operator,
                Operator::SetSubtract => operator,
                Operator::SetMultiply => operator,
                Operator::SetDivide => operator,
                Operator::SetModulo => operator,
                Operator::SetExponent => operator,
                _ => {
                    return Err(ParserError(
                        "Unexpected operator in update operation".to_string(),
                        pos.clone(),
                    ))
                }
            },
            Some(Token(_, pos)) => {
                return Err(ParserError(
                    "Unexpected token in update operation".to_string(),
                    pos.clone(),
                ))
            }
            None => {
                return Err(ParserError(
                    "Expected set operator or set operator variant".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        }
        .clone();

        let value = parse_operand_block(token_stream, vec![TokenType::Terminate], file)?;

        return Ok(Term::UpdateVar {
            var,
            set_operator,
            value,
        });
    }

    // Parse if block
    if let Token(TokenType::KeyWord(KeyWord::If), _) = lead_token {
        let conditional = parse_operand_block(
            token_stream,
            vec![TokenType::Operator(Operator::OpenBlock)],
            file,
        )?;
        token_stream.back();
        let block = parse_block(token_stream, file)?;

        let else_block = match token_stream.advance() {
            Some(Token(TokenType::KeyWord(KeyWord::Else), _)) => match token_stream.advance() {
                Some(token) => match token {
                    Token(TokenType::KeyWord(KeyWord::If), _) => TermBlock {
                        terms: vec![parse_term(token.clone(), token_stream, file)?],
                    },
                    Token(TokenType::Operator(Operator::OpenBlock), _) => {
                        token_stream.back();
                        parse_block(token_stream, file)?
                    }
                    _ => {
                        return Err(ParserError(
                            "Unexpected token in else declaration".to_string(),
                            token.1.clone(),
                        ))
                    }
                },
                None => {
                    return Err(ParserError(
                        "Expected else body".to_string(),
                        FileLocation::End { file: file.clone() },
                    ))
                }
            },
            _ => {
                token_stream.back();
                TermBlock {
                    terms: Vec::<Term>::new(),
                }
            }
        };

        return Ok(Term::If {
            conditional,
            block,
            else_block,
        });
    }

    // Parse loop
    if let Token(TokenType::KeyWord(KeyWord::Loop), _) = lead_token {
        let counter = match token_stream.advance().cloned() {
            Some(op) => match op.0 {
                TokenType::Identity(id) => id,
                _ => {
                    return Err(ParserError(
                        "Unexpected token in place of loop counter name".to_string(),
                        op.1,
                    ))
                }
            },
            None => {
                return Err(ParserError(
                    "Expected loop counter name".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        };

        // Burn var conditional seperator
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Colon), _)) => {}
            Some(token) => {
                return Err(ParserError(
                    "Unexpected token in loop signiture".to_string(),
                    token.1.clone(),
                ))
            }

            None => {
                return Err(ParserError(
                    "Premeture end to loop definition".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        };

        let conditional = parse_operand_block(
            token_stream,
            vec![TokenType::Operator(Operator::OpenBlock)],
            file,
        )?;
        token_stream.back();

        let block = parse_block(token_stream, file)?;

        return Ok(Term::Loop {
            counter,
            conditional,
            block,
        });
    }

    // Parse break
    if let Token(TokenType::KeyWord(KeyWord::Break), _) = lead_token {
        return match token_stream.advance() {
            Some(token) => match token.0 {
                TokenType::Terminate => Ok(Term::Break(token.1.clone())),
                _ => {
                    return Err(ParserError(
                        "Unexpected token at after break".to_string(),
                        token.1.clone(),
                    ))
                }
            },
            None => {
                return Err(ParserError(
                    "Expected line terminator".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        };
    }

    // Parse continue
    if let Token(TokenType::KeyWord(KeyWord::Continue), _) = lead_token {
        return match token_stream.advance() {
            Some(token) => match token.0 {
                TokenType::Terminate => Ok(Term::Continue(token.1.clone())),
                _ => {
                    return Err(ParserError(
                        "Unexpected token at after continue".to_string(),
                        token.1.clone(),
                    ))
                }
            },
            None => {
                return Err(ParserError(
                    "Expected line terminator".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        };
    }

    // Parse call
    if let Token(TokenType::KeyWord(KeyWord::Call), _) = lead_token {
        return Ok(Term::Call {
            value: parse_operand_block(token_stream, vec![TokenType::Terminate], file)?,
        });
    }

    return Err(ParserError(
        "Unrecognized term".to_string(),
        lead_token.1.clone(),
    ));
}

// Parse function
fn parse_func(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Function, ParserError> {
    // Get return type of function
    let returntype = parse_type::parse_type(token_stream, file)?;

    // Get identity of function
    let (name, loc) = match token_stream.advance().cloned() {
        Some(op) => match op.0 {
            TokenType::Identity(id) => (id, op.1),
            _ => {
                return Err(ParserError(
                    "Unexpected token instead of function name".to_string(),
                    op.1,
                ))
            }
        },
        None => {
            return Err(ParserError(
                "Expected function name".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    // Burn identity arg separator in function signature
    let args = match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::Colon), _)) => true,
        Some(Token(TokenType::Operator(Operator::OpenBlock), _)) => false,
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in function signiture".to_string(),
                token.1.clone(),
            ))
        }
        None => {
            return Err(ParserError(
                "Premature end to function signiture".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    // Get normal args of function
    let args = match args {
        true => {
            // Create arg list
            let mut args = Vec::<VarSigniture>::new();

            // Loop until end of arg list found
            loop {
                //  Ensure token exits else return syntax error
                let token = match token_stream.advance() {
                    Some(token) => token,
                    None => {
                        return Err(ParserError(
                            "Expected function body".to_string(),
                            FileLocation::End { file: file.clone() },
                        ))
                    }
                };

                // If state of function block found exit loop
                if let Token(TokenType::Operator(Operator::OpenBlock), _) = token {
                    break;
                }

                // Roll token stream back to start of arg declaration
                token_stream.back();

                let var_sig = parse_type::parse_var_sig(token_stream, file)?;

                // Parse argument and add to arg list
                args.push(var_sig.clone());

                // Check if arg list continues or start of block found else return syntax error
                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::OpenBlock), _)) => break,
                    Some(Token(TokenType::Operator(Operator::Comma), _)) => continue,
                    Some(token) => {
                        return Err(ParserError(
                            "Unexpected token in function signiture".to_string(),
                            token.1.clone(),
                        ))
                    }
                    None => {
                        return Err(ParserError(
                            "Expected function body".to_string(),
                            FileLocation::End { file: file.clone() },
                        ))
                    }
                }
            }

            // return arg list
            args
        }
        false => Vec::new(),
    };

    // Roll back to beginning of function block
    token_stream.back();

    // Get function block
    let block = parse_block(token_stream, file)?;

    // Add function to term array
    return Ok(Function {
        name: name.to_owned(),
        returntype,
        args,
        block,
        loc,
    });
}

// Parse struct
fn parse_struct(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Struct, ParserError> {
    let (name, loc) = match token_stream.advance().cloned() {
        Some(op) => match op.0 {
            TokenType::Identity(id) => (id, op.1),
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of class name".to_string(),
                    (op.1).clone(),
                ))
            }
        },
        None => {
            return Err(ParserError(
                "Expected class name".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    // Get class block open
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenBlock), _)) => {}
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in class definition".to_string(),
                token.1.clone(),
            ))
        }
        None => {
            return Err(ParserError(
                "Premature end to class definition".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    let mut properties = Vec::<VarSigniture>::new();
    let mut methods = Vec::<Function>::new();
    loop {
        if let Some(Token(TokenType::Operator(Operator::CloseBlock), _)) = token_stream.advance() {
            break;
        }

        token_stream.back();

        match token_stream.advance() {
            Some(token) => match token {
                Token(TokenType::KeyWord(KeyWord::Var), _) => {
                    let var_sig = parse_var_sig(token_stream, file)?;

                    // Check for terminating char
                    match token_stream.advance() {
                        Some(Token(TokenType::Terminate, _)) => {}
                        Some(token) => {
                            return Err(ParserError(
                                "Unexpected token in property definition. You can only initialize a delcaration statement inside a function.".to_string(),
                                token.1.clone(),
                            ))
                        }
                        None => {
                            return Err(ParserError(
                                "Expected line terminator".to_string(),
                                FileLocation::End { file: file.clone() },
                            ))
                        }
                    };

                    properties.push(var_sig);
                }
                Token(TokenType::KeyWord(KeyWord::Func), _) => {
                    let func = parse_func(token_stream, file)?;
                    methods.push(func);
                }
                _ => {
                    return Err(ParserError(
                        "Unexpected token within class block".to_string(),
                        token.1.clone(),
                    ))
                }
            },
            None => {
                return Err(ParserError(
                    "Expected class block close".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        }
    }

    return Ok(Struct {
        name,
        properties,
        methods,
        loc,
    });
}

// Parse code within block
fn parse_block(token_stream: &mut TokenStream, file: &PathBuf) -> Result<TermBlock, ParserError> {
    // Check for block open
    let mut block = match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenBlock), _)) => TermBlock {
            terms: Vec::<Term>::new(),
        },
        Some(token) => return Err(ParserError("Expected block".to_string(), token.1.clone())),
        None => {
            return Err(ParserError(
                "Expected block".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    let TermBlock { ref mut terms } = block;

    // Parse terms until end of block found
    while let Some(token) = token_stream.advance().cloned() {
        // Break loop if end of block found
        if let Token(TokenType::Operator(Operator::CloseBlock), _) = token {
            break;
        }

        // Parse term
        let term = parse_term(token, token_stream, file)?;
        terms.push(term);
    }

    match token_stream.current() {
        Some(Token(TokenType::Operator(Operator::CloseBlock), _pos)) => return Ok(block),
        Some(Token(_, pos)) => {
            return Err(ParserError(
                "Unexpected token at block closing".to_string(),
                pos.clone(),
            ))
        }
        None => {
            return Err(ParserError(
                "Expected block close".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    }
}

fn parse_program(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Program, ErrorType> {
    let mut program = Program {
        structs: Vec::new(),
        functions: Vec::new(),
    };

    while let Some(token) = token_stream.advance().cloned() {
        match token.0 {
            TokenType::KeyWord(keyword) => match keyword {
                KeyWord::Struct => program
                    .structs
                    .push(match parse_struct(token_stream, file) {
                        Ok(ok) => ok,
                        Err(err) => return Err(ErrorType::Parser(err)),
                    }),
                KeyWord::Func => program
                    .functions
                    .push(match parse_func(token_stream, file) {
                        Ok(ok) => ok,
                        Err(err) => return Err(ErrorType::Parser(err)),
                    }),
                KeyWord::Import => {
                    let mut objects = Vec::new();
                    while let Some(token) = token_stream.advance() {
                        match &token.0 {
                            TokenType::Identity(id) => {
                                objects.push(id.clone());

                                match token_stream.advance() {
                                    Some(token) => match token.0 {
                                        TokenType::Operator(Operator::Comma) => continue,
                                        TokenType::KeyWord(KeyWord::Of) => break,
                                        _ => {
                                            return Err(ErrorType::Parser(ParserError(
                                                "Unexpected token in import.".to_owned(),
                                                token.1.clone(),
                                            )))
                                        }
                                    },
                                    None => {
                                        return Err(ErrorType::Parser(ParserError(
                                            "Expected import file.".to_owned(),
                                            FileLocation::End { file: file.clone() },
                                        )))
                                    }
                                }
                            }
                            _ => {
                                return Err(ErrorType::Parser(ParserError(
                                    "Expected object name to import.".to_owned(),
                                    token.1.clone(),
                                )))
                            }
                        }
                    }

                    let file_token = match token_stream.advance() {
                        Some(t) => t,
                        None => {
                            return Err(ErrorType::Parser(ParserError(
                                "Expected string after import.".to_string(),
                                FileLocation::End { file: file.clone() },
                            )))
                        }
                    }
                    .clone();
                    let file_string = match file_token {
                        Token(TokenType::String(file), _) => file.to_owned(),
                        _ => {
                            return Err(ErrorType::Parser(ParserError(
                                "Expected string after import.".to_string(),
                                file_token.1.clone(),
                            )))
                        }
                    };

                    let path = file.parent().unwrap().join(PathBuf::from(file_string));
                    match token_stream.advance() {
                        Some(Token(TokenType::Terminate, _)) => {}
                        Some(t) => {
                            return Err(ErrorType::Parser(ParserError(
                                "Expected line terminator".to_string(),
                                t.1.clone(),
                            )))
                        }
                        None => {
                            return Err(ErrorType::Parser(ParserError(
                                "Expected line terminator".to_string(),
                                FileLocation::End { file: file.clone() },
                            )))
                        }
                    }

                    let module = {
                        let mut module = match fs::read_to_string(&path) {
                            Ok(ok) => ok,
                            Err(_) => {
                                return Err(ErrorType::Parser(ParserError(
                                    "Cannot read mod input file".to_string(),
                                    file_token.1.clone(),
                                )))
                            }
                        };
                        module.push(' ');
                        module
                    };
                    let lex_out = match lexer::lex(
                        &module,
                        false,
                        &path,
                        &format!("{}::", path.to_string_lossy()),
                        &objects,
                    ) {
                        Ok(ok) => ok,
                        Err(err) => return Err(ErrorType::Lexer(err)),
                    };
                    let mut parse_out = parse(lex_out, &path)?;

                    program.structs.append(&mut parse_out.structs);
                    program.functions.append(&mut parse_out.functions);
                }
                _ => {
                    return Err(ErrorType::Parser(ParserError(
                        format!("Invalid keyword, {}, in program namespace", keyword),
                        token.1,
                    )))
                }
            },
            _ => {
                return Err(ErrorType::Parser(ParserError(
                    format!("Invalid token, {}, in program namespace", token.0),
                    token.1,
                )))
            }
        }
    }

    return Ok(program);
}

// Parse a Token Vector
pub fn parse(input: Vec<Token>, file: &PathBuf) -> Result<Program, ErrorType> {
    let mut token_stream = TokenStream::new(input);
    let _program_prelude = match token_stream.current() {
        Some(Token(TokenType::String(string), _)) => string,
        Some(token) => {
            let mut loc = token.1.clone();
            if let FileLocation::Loc {
                start_line,
                end_line,
                start_col,
                mut end_col,
                ..
            } = loc
            {
                end_col = 1;
                loc = FileLocation::Loc {
                    file: file.clone(),
                    start_line,
                    end_line,
                    start_col,
                    end_col,
                }
            }
            return Err(ErrorType::Parser(ParserError(
                "No program prelude string found".to_string(),
                loc,
            )));
        }
        None => {
            return Err(ErrorType::Parser(ParserError(
                "Program file empty".to_string(),
                FileLocation::None,
            )))
        }
    };
    return parse_program(&mut token_stream, file);
}
