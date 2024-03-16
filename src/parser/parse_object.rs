use crate::{
    errors::{FileLocation, ParserError},
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{
    parse_operand_block::{parse_operand_block, OperandExpression},
    parse_type::parse_type,
    Call, Object, ObjectCreate, ObjectType, TokenStream, Type,
};

// Parse function call
fn parse_call(token_stream: &mut TokenStream) -> Result<Call, ParserError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenParen), _)) => {}
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in call arguments".to_string(),
                (token.1.clone()),
            ))
        }
        None => {
            return Err(ParserError(
                "Expected start of call arguments".to_string(),
                FileLocation::EOF,
            ))
        }
    };

    let typeargs = match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::Less), _)) => {
            let mut typeargs = Vec::<Type>::new();
            loop {
                if let Some(Token(TokenType::Operator(Operator::Greater), _)) =
                    token_stream.advance()
                {
                    break;
                }

                token_stream.back();
                typeargs.push(parse_type(token_stream)?);

                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Comma), _)) => {}
                    _ => {
                        token_stream.back();
                    }
                };
            }

            typeargs
        }

        Some(_) => {
            token_stream.back();
            Vec::<Type>::new()
        }
        None => {
            return Err(ParserError(
                "Expected close of call arguments".to_string(),
                FileLocation::EOF,
            ))
        }
    };

    let mut args = Vec::<OperandExpression>::new();
    loop {
        // Add clause to check for empty list: ()
        if let Some(Token(TokenType::Operator(Operator::CloseParen), _)) = token_stream.advance() {
            break;
        }

        token_stream.back();

        args.push(parse_operand_block(
            token_stream,
            vec![
                TokenType::Operator(Operator::CloseParen),
                TokenType::Operator(Operator::Comma),
            ],
        )?);

        if let Some(Token(TokenType::Operator(Operator::CloseParen), _)) = token_stream.current() {
            break;
        }
    }

    return Ok(Call { typeargs, args });
}

// Parse identity object Nonpeekable, Noncallable
pub fn parse_object(token_stream: &mut TokenStream) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    return Err(ParserError(
                        "Cannot peek, call, or index identity at this location".to_string(),
                        (pos.clone()),
                    ));
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    (token.1.clone()),
                ))
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::EOF,
            ))
        }
    }
}

// Parse identity object Peekable, Noncallable
pub fn parse_object_peekable(token_stream: &mut TokenStream) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    match token_stream.advance() {
                        Some(Token(TokenType::Operator(Operator::OpenBracket), _)) => {
                            return Err(ParserError(
                                "Cannot call identity at this location".to_string(),
                                (pos.clone()),
                            ))
                        }
                        Some(Token(TokenType::Operator(Operator::OpenParen), _)) => {
                            return Err(ParserError(
                                "Cannot index identity at this location".to_string(),
                                (pos.clone()),
                            ))
                        }
                        _ => {
                            token_stream.back();
                            return Ok(Object {
                                kind: ObjectType::Identity(id),
                                sub: Some(Box::new(parse_object_peekable(token_stream)?)),
                            });
                        }
                    }
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    (token.1.clone()),
                ));
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::EOF,
            ))
        }
    }
}

// Parse identity object Peekable, Callable
pub fn parse_object_peekable_callable(
    token_stream: &mut TokenStream,
) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                    return Ok(Object {
                        kind: ObjectType::Identity(id),
                        sub: Some(Box::new(parse_object_peekable_callable(token_stream)?)),
                    });
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            TokenType::Operator(Operator::OpenParen) => {
                token_stream.back();
                let call = parse_call(token_stream)?;
                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                        return Ok(Object {
                            kind: ObjectType::Call(call),
                            sub: Some(Box::new(parse_object_peekable_callable(token_stream)?)),
                        });
                    }
                    _ => {
                        token_stream.back();
                        return Ok(Object {
                            kind: ObjectType::Call(call),
                            sub: None,
                        });
                    }
                }
            }
            TokenType::Operator(Operator::OpenBracket) => {
                let index = parse_operand_block(
                    token_stream,
                    vec![TokenType::Operator(Operator::CloseBracket)],
                )?;

                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                        return Ok(Object {
                            kind: ObjectType::Index(Box::new(index)),
                            sub: Some(Box::new(parse_object_peekable_callable(token_stream)?)),
                        });
                    }
                    _ => {
                        token_stream.back();
                        Ok(Object {
                            kind: ObjectType::Index(Box::new(index)),
                            sub: None,
                        })
                    }
                }
            }
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    (token.1.clone()),
                ));
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::EOF,
            ))
        }
    }
}

pub fn parse_object_create(token_stream: &mut TokenStream) -> Result<ObjectCreate, ParserError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::New), _)) => {}
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in object creation".to_string(),
                (token.1.clone()),
            ))
        }
        None => {
            return Err(ParserError(
                "Expected creation operator".to_string(),
                FileLocation::EOF,
            ))
        }
    };

    let call = parse_call(token_stream)?;
    let type_ = parse_type(token_stream)?;

    return Ok(ObjectCreate {
        kind: type_,
        args: call,
    });
}
