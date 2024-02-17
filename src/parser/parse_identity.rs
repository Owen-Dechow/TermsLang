use crate::{
    errors::SyntaxError,
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{
    parse_operand_block::{parse_operand_block, OperandExpression},
    parse_type::parse_type,
    Call, Index, Object, ObjectType, TokenStream, Type,
};

// Parse function call
fn parse_call(token_stream: &mut TokenStream) -> Result<Call, SyntaxError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenParen), ..)) => {}
        Some(token) => {
            return Err(SyntaxError(
                "Unexpected token in call arguments".to_string(),
                Some(token.1.clone()),
            ))
        }
        None => {
            return Err(SyntaxError(
                "Expected start of call arguments".to_string(),
                None,
            ))
        }
    };

    let typeargs = match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::Less), ..)) => {
            let mut typeargs = Vec::<Type>::new();
            loop {
                if let Some(Token(TokenType::Operator(Operator::Greater), ..)) =
                    token_stream.advance()
                {
                    break;
                }

                token_stream.back();
                typeargs.push(parse_type(token_stream)?);

                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Comma), ..)) => {}
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
            return Err(SyntaxError(
                "Expected close of call arguments".to_string(),
                None,
            ))
        }
    };

    let mut args = Vec::<OperandExpression>::new();
    loop {
        // Add clause to check for empty list: ()
        if let Some(Token(TokenType::Operator(Operator::CloseParen), ..)) = token_stream.advance() {
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

        if let Some(Token(TokenType::Operator(Operator::CloseParen), ..)) = token_stream.current() {
            break;
        }
    }

    return Ok(Call { typeargs, args });
}

// Parse function call
fn parse_index(token_stream: &mut TokenStream) -> Result<Index, SyntaxError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenBracket), ..)) => {}
        Some(token) => {
            return Err(SyntaxError(
                "Unexpected token in index".to_string(),
                Some(token.1.clone()),
            ))
        }
        None => return Err(SyntaxError("Expected start of index".to_string(), None)),
    };

    let mut indexes = Vec::<OperandExpression>::new();
    loop {
        // Add clause to check for empty list: ()
        if let Some(Token(TokenType::Operator(Operator::CloseBracket), ..)) = token_stream.advance()
        {
            break;
        }

        token_stream.back();

        indexes.push(parse_operand_block(
            token_stream,
            vec![
                TokenType::Operator(Operator::CloseBracket),
                TokenType::Operator(Operator::Comma),
            ],
        )?);

        if let Some(Token(TokenType::Operator(Operator::CloseBracket), ..)) = token_stream.current()
        {
            break;
        }
    }

    return Ok(Index(indexes));
}

// Parse identity object Nonpeekable, Noncallable
pub fn parse_object(token_stream: &mut TokenStream) -> Result<Object, SyntaxError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(..) => match token_stream.advance() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    return Err(SyntaxError(
                        "Cannot peek, call, or index identity at this location".to_string(),
                        Some(pos.clone()),
                    ))
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(token.clone()),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(SyntaxError(
                    "Unexpected token in place of identity".to_string(),
                    Some(token.1.clone()),
                ))
            }
        },
        None => return Err(SyntaxError("Expected identity".to_string(), None)),
    }
}

// Parse identity object Peekable, Noncallable
pub fn parse_object_peekable(token_stream: &mut TokenStream) -> Result<Object, SyntaxError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(..) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    match token_stream.advance() {
                        Some(Token(TokenType::Operator(Operator::OpenBracket), ..)) => {
                            return Err(SyntaxError(
                                "Cannot call identity at this location".to_string(),
                                Some(pos.clone()),
                            ))
                        }
                        Some(Token(TokenType::Operator(Operator::OpenParen), ..)) => {
                            return Err(SyntaxError(
                                "Cannot index identity at this location".to_string(),
                                Some(pos.clone()),
                            ))
                        }
                        _ => {
                            token_stream.back();
                            return Ok(Object {
                                kind: ObjectType::Identity(token.clone()),
                                sub: Some(Box::new(parse_object_peekable(token_stream)?)),
                            });
                        }
                    }
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(token.clone()),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(SyntaxError(
                    "Unexpected token in place of identity".to_string(),
                    Some(token.1.clone()),
                ))
            }
        },
        None => return Err(SyntaxError("Expected identity".to_string(), None)),
    }
}

// Parse identity object Peekable, Callable
pub fn parse_object_peekable_callable(
    token_stream: &mut TokenStream,
) -> Result<Object, SyntaxError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(..) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), ..)) => {
                    return Ok(Object {
                        kind: ObjectType::Identity(token.clone()),
                        sub: Some(Box::new(parse_object_peekable_callable(token_stream)?)),
                    });
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        kind: ObjectType::Identity(token.clone()),
                        sub: None,
                    });
                }
            },
            TokenType::Operator(Operator::OpenParen) => {
                token_stream.back();
                let call = parse_call(token_stream)?;
                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), ..)) => {
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
                token_stream.back();
                let indexing = parse_index(token_stream)?;
                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), ..)) => {
                        return Ok(Object {
                            kind: ObjectType::Index(indexing),
                            sub: Some(Box::new(parse_object_peekable_callable(token_stream)?)),
                        });
                    }
                    _ => {
                        token_stream.back();
                        return Ok(Object {
                            kind: ObjectType::Index(indexing),
                            sub: None,
                        });
                    }
                }
            }
            _ => {
                return Err(SyntaxError(
                    "Unexpected token in place of identity".to_string(),
                    Some(token.1.clone()),
                ))
            }
        },
        None => return Err(SyntaxError("Expected identity".to_string(), None)),
    }
}