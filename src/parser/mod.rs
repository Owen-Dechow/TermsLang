use crate::{
    errors::{FileLocation, ParserError},
    lexer::tokens::{KeyWord, Operator, Token, TokenType},
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

    fn add_lead(&mut self) {
        self.tokens.insert(
            0,
            Token(
                TokenType::Terminate,
                FileLocation::Loc {
                    start_line: 0,
                    end_line: 0,
                    start_col: 0,
                    end_col: 0,
                },
            ),
        );
    }
}

#[derive(Debug, Clone)]
pub enum Array {
    Matrix(Index),
    Normal,
    Not,
}

#[derive(Debug, Clone)]
pub enum Type {
    Array(Box<Type>),
    Object {
        object: Object,
        associated_types: Vec<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct Object {
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
pub struct Index(Vec<OperandExpression>);

#[derive(Debug, Clone)]
pub struct VarSigniture {
    pub identity: String,
    pub argtype: Type,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub func: Term,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub enum Term {
    Block {
        terms: Vec<Term>,
    },
    Func {
        name: String,
        returntype: Type,
        args: Vec<VarSigniture>,
        block: Box<Term>,
    },
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
        block: Box<Term>,
        else_block: Box<Term>,
    },
    Loop {
        counter: String,
        conditional: OperandExpression,
        block: Box<Term>,
    },
    ReadLn {
        var: Object,
    },
    Break,
    Continue,
    Call {
        value: OperandExpression,
    },
    Struct {
        name: String,
        properties: Vec<VarSigniture>,
        methods: Vec<Method>,
    },
}

// Parse single term
fn parse_term(lead_token: Token, token_stream: &mut TokenStream) -> Result<Term, ParserError> {
    // Parse function
    if let Token(TokenType::KeyWord(KeyWord::Func), _) = lead_token {
        // Get return type of function
        let returntype = parse_type::parse_type(token_stream)?;

        // Get identity of function
        let name = match token_stream.advance().cloned() {
            Some(op) => match op.0 {
                TokenType::Identity(id) => id,
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
                    FileLocation::End,
                ))
            }
        };

        // Burn identity arg separator in function signiture
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Colon), _)) => {}
            Some(token) => {
                return Err(ParserError(
                    "Unexpected token in function signiture".to_string(),
                    token.1.clone(),
                ))
            }
            None => {
                return Err(ParserError(
                    "Premature end to function signiture".to_string(),
                    FileLocation::End,
                ))
            }
        };

        // Get normal args of function
        let args = {
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
                            FileLocation::End,
                        ))
                    }
                };

                // If state of function block found exit loop
                if let Token(TokenType::Operator(Operator::OpenBlock), _) = token {
                    break;
                }

                // Roll token stream back to start of arg declaration
                token_stream.back();

                let var_sig = parse_type::parse_var_sig(token_stream)?;

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
                            FileLocation::End,
                        ))
                    }
                }
            }

            // return arg list
            args
        };

        // Roll back to beginning of function block
        token_stream.back();

        // Get function block
        let block = Box::new(parse_block(token_stream, false)?);

        // Add function to term array
        return Ok(Term::Func {
            name: name.to_owned(),
            returntype,
            args,
            block,
        });
    }

    // Parse print
    if let Token(TokenType::KeyWord(KeyWord::Print), _) = lead_token {
        return Ok(Term::Print {
            ln: false,
            operand_block: parse_operand_block(token_stream, vec![TokenType::Terminate])?,
        });
    }

    // Parse println
    if let Token(TokenType::KeyWord(KeyWord::PrintLn), _) = lead_token {
        return Ok(Term::Print {
            ln: true,
            operand_block: parse_operand_block(token_stream, vec![TokenType::Terminate])?,
        });
    }

    // Parse var declaration
    if let Token(TokenType::KeyWord(KeyWord::Var), _) = lead_token {
        let vartype = parse_type(token_stream)?;
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
                    FileLocation::End,
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
            None => todo!(),
        }

        let value = parse_operand_block(token_stream, vec![TokenType::Terminate])?;

        return Ok(Term::DeclareVar {
            name: name.to_owned(),
            vartype,
            value,
        });
    }

    // Parse return
    if let Token(TokenType::KeyWord(KeyWord::Return), _) = lead_token {
        return Ok(Term::Return {
            value: parse_operand_block(token_stream, vec![TokenType::Terminate])?,
        });
    }

    // Parse var update
    if let Token(TokenType::KeyWord(KeyWord::UpdateVar), _) = lead_token {
        let var = parse_object_peekable(token_stream)?;

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
                    FileLocation::End,
                ))
            }
        }
        .clone();

        let value = parse_operand_block(token_stream, vec![TokenType::Terminate])?;

        return Ok(Term::UpdateVar {
            var,
            set_operator,
            value,
        });
    }

    // Parse if block
    if let Token(TokenType::KeyWord(KeyWord::If), _) = lead_token {
        let conditional =
            parse_operand_block(token_stream, vec![TokenType::Operator(Operator::OpenBlock)])?;
        token_stream.back();
        let block = parse_block(token_stream, false)?;

        let else_block = match token_stream.advance() {
            Some(Token(TokenType::KeyWord(KeyWord::Else), _)) => match token_stream.advance() {
                Some(token) => match token {
                    Token(TokenType::KeyWord(KeyWord::If), _) => {
                        parse_term(token.clone(), token_stream)?
                    }
                    Token(TokenType::Operator(Operator::OpenBlock), _) => {
                        token_stream.back();
                        parse_block(token_stream, false)?
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
                        FileLocation::End,
                    ))
                }
            },
            _ => {
                token_stream.back();
                Term::Block {
                    terms: Vec::<Term>::new(),
                }
            }
        };

        return Ok(Term::If {
            conditional,
            block: Box::new(block),
            else_block: Box::new(else_block),
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
                    FileLocation::End,
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
                    FileLocation::End,
                ))
            }
        };

        let conditional =
            parse_operand_block(token_stream, vec![TokenType::Operator(Operator::OpenBlock)])?;
        token_stream.back();

        let block = parse_block(token_stream, false)?;

        return Ok(Term::Loop {
            counter,
            conditional,
            block: Box::new(block),
        });
    }

    // Parse break
    if let Token(TokenType::KeyWord(KeyWord::Break), _) = lead_token {
        return match token_stream.advance() {
            Some(token) => match token.0 {
                TokenType::Terminate => Ok(Term::Break),
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
                    FileLocation::End,
                ))
            }
        };
    }

    // Parse continue
    if let Token(TokenType::KeyWord(KeyWord::Continue), _) = lead_token {
        return match token_stream.advance() {
            Some(token) => match token.0 {
                TokenType::Terminate => Ok(Term::Continue),
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
                    FileLocation::End,
                ))
            }
        };
    }

    // Parse call
    if let Token(TokenType::KeyWord(KeyWord::Call), _) = lead_token {
        return Ok(Term::Call {
            value: parse_operand_block(token_stream, vec![TokenType::Terminate])?,
        });
    }

    // Parse readln
    if let Token(TokenType::KeyWord(KeyWord::ReadLn), _) = lead_token {
        let var = parse_object_peekable(token_stream)?;

        return match token_stream.advance() {
            Some(token) => match token.0 {
                TokenType::Terminate => Ok(Term::ReadLn { var }),
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
                    FileLocation::End,
                ))
            }
        };
    }

    // Parse class
    if let Token(TokenType::KeyWord(KeyWord::Struct), _) = lead_token {
        let name = match token_stream.advance().cloned() {
            Some(op) => match op.0 {
                TokenType::Identity(id) => id,
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
                    FileLocation::End,
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
                    FileLocation::End,
                ))
            }
        };

        let mut methods = Vec::<Method>::new();
        let mut properties = Vec::<VarSigniture>::new();
        loop {
            if let Some(Token(TokenType::Operator(Operator::CloseBlock), _)) =
                token_stream.advance()
            {
                break;
            }

            token_stream.back();

            match token_stream.advance() {
                Some(token) => match token {
                    Token(TokenType::KeyWord(KeyWord::Static), _) => {
                        // Ensure func keyword
                        let func_token = match token_stream.advance() {
                            Some(token) => match token {
                                Token(TokenType::KeyWord(KeyWord::Func), _) => token,
                                _ => {
                                    return Err(ParserError(
                                        "Unexpected token in static func definition".to_string(),
                                        FileLocation::End,
                                    ))
                                }
                            },
                            _ => {
                                return Err(ParserError(
                                    "Expected func definition".to_string(),
                                    FileLocation::End,
                                ))
                            }
                        };

                        let function = parse_term(func_token.clone(), token_stream)?;
                        methods.push(Method {
                            func: function,
                            is_static: true,
                        })
                    }
                    Token(TokenType::KeyWord(KeyWord::Func), _) => {
                        let function = parse_term(token.clone(), token_stream)?;
                        methods.push(Method {
                            func: function,
                            is_static: false,
                        })
                    }
                    Token(TokenType::KeyWord(KeyWord::Var), _) => {
                        let var_sig = parse_var_sig(token_stream)?;

                        // Check for terminating char
                        match token_stream.advance() {
                            Some(Token(TokenType::Terminate, _)) => {}
                            Some(token) => {
                                return Err(ParserError(
                                    "Unexpected token in property definition".to_string(),
                                    token.1.clone(),
                                ))
                            }
                            None => {
                                return Err(ParserError(
                                    "Expected line terminator".to_string(),
                                    FileLocation::End,
                                ))
                            }
                        };

                        properties.push(var_sig);
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
                        FileLocation::End,
                    ))
                }
            }
        }

        return Ok(Term::Struct {
            name,
            properties,
            methods,
        });
    }

    return Err(ParserError(
        "Unrecognized term".to_string(),
        lead_token.1.clone(),
    ));
}

// Parse code within block
fn parse_block(
    token_stream: &mut TokenStream,
    omit_block_tokens: bool,
) -> Result<Term, ParserError> {
    // Check for block open
    let mut block = {
        if omit_block_tokens {
            Term::Block {
                terms: Vec::<Term>::new(),
            }
        } else {
            match token_stream.advance() {
                Some(Token(TokenType::Operator(Operator::OpenBlock), _)) => Term::Block {
                    terms: Vec::<Term>::new(),
                },
                Some(token) => {
                    return Err(ParserError("Expected block".to_string(), token.1.clone()))
                }
                None => return Err(ParserError("Expected block".to_string(), FileLocation::End)),
            }
        }
    };

    // Extract block variables as ref mut
    if let Term::Block { ref mut terms } = block {
        // Parse terms until end of block found
        while let Some(t) = token_stream.advance().cloned() {
            // Break loop if end of block found
            if !omit_block_tokens {
                if let Token(TokenType::Operator(Operator::CloseBlock), _) = t {
                    break;
                }
            }

            // Parse term
            let term = parse_term(t, token_stream)?;
            terms.push(term);
        }
    };

    if omit_block_tokens {
        return Ok(block);
    } else {
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
                    FileLocation::End,
                ))
            }
        }
    }
}

// Parse a Token Vector
pub fn parse(input: Vec<Token>) -> Result<Term, ParserError> {
    // Create token stream
    let mut token_stream = TokenStream::new(input);
    token_stream.add_lead();

    // Parse and return the program block
    return parse_block(&mut token_stream, true);
}
