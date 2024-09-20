use std::path::PathBuf;

use crate::{
    errors::{FileLocation, ParserError},
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{
    parse_operand_block::{parse_operand_block, OperandExpression},
    parse_type::parse_type,
    Call, Object, ObjectCreate, ObjectType, TokenStream,
};

// Parse function call
fn parse_call(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Call, ParserError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::OpenParen), _)) => {}
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in call arguments".to_string(),
                token.1.clone(),
            ))
        }
        None => {
            return Err(ParserError(
                "Expected start of call arguments".to_string(),
                FileLocation::End { file: file.clone() },
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
            file,
        )?);

        if let Some(Token(TokenType::Operator(Operator::CloseParen), _)) = token_stream.current() {
            break;
        }
    }

    return Ok(Call { args });
}

// Parse identity object Nonpeekable, Noncallable
pub fn parse_object(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    return Err(ParserError(
                        "Cannot peek, call, or index identity at this location".to_string(),
                        pos.clone(),
                    ));
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        loc: token.1,
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    token.1.clone(),
                ))
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    }
}

// Parse identity object Peekable, Noncallable
pub fn parse_object_peekable(
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), pos)) => {
                    match token_stream.advance() {
                        Some(Token(TokenType::Operator(Operator::OpenBracket), _)) => {
                            return Err(ParserError(
                                "Cannot call identity at this location".to_string(),
                                pos.clone(),
                            ))
                        }
                        Some(Token(TokenType::Operator(Operator::OpenParen), _)) => {
                            return Err(ParserError(
                                "Cannot index identity at this location".to_string(),
                                pos.clone(),
                            ))
                        }
                        _ => {
                            token_stream.back();
                            return Ok(Object {
                                loc: token.1,
                                kind: ObjectType::Identity(id),
                                sub: Some(Box::new(parse_object_peekable(token_stream, file)?)),
                            });
                        }
                    }
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        loc: token.1,
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    token.1.clone(),
                ));
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    }
}

// Parse identity object Peekable, Callable
pub fn parse_object_peekable_callable(
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<Object, ParserError> {
    match token_stream.advance().cloned() {
        Some(token) => match token.0 {
            TokenType::Identity(id) => match token_stream.advance().cloned() {
                Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                    return Ok(Object {
                        loc: token.1,
                        kind: ObjectType::Identity(id),
                        sub: Some(Box::new(parse_object_peekable_callable(
                            token_stream,
                            file,
                        )?)),
                    });
                }
                _ => {
                    token_stream.back();
                    return Ok(Object {
                        loc: token.1,
                        kind: ObjectType::Identity(id),
                        sub: None,
                    });
                }
            },
            TokenType::Operator(Operator::OpenParen) => {
                token_stream.back();
                let call = parse_call(token_stream, file)?;
                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                        return Ok(Object {
                            loc: token.1,
                            kind: ObjectType::Call(call),
                            sub: Some(Box::new(parse_object_peekable_callable(
                                token_stream,
                                file,
                            )?)),
                        });
                    }
                    _ => {
                        token_stream.back();
                        return Ok(Object {
                            loc: token.1,
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
                    file,
                )?;

                match token_stream.advance() {
                    Some(Token(TokenType::Operator(Operator::Dot), _)) => {
                        return Ok(Object {
                            loc: token.1,
                            kind: ObjectType::Index(Box::new(index)),
                            sub: Some(Box::new(parse_object_peekable_callable(
                                token_stream,
                                file,
                            )?)),
                        });
                    }
                    _ => {
                        token_stream.back();
                        Ok(Object {
                            loc: token.1,
                            kind: ObjectType::Index(Box::new(index)),
                            sub: None,
                        })
                    }
                }
            }
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of identity".to_string(),
                    token.1.clone(),
                ));
            }
        },
        None => {
            return Err(ParserError(
                "Expected identity".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    }
}

pub fn parse_object_create(
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<ObjectCreate, ParserError> {
    match token_stream.advance() {
        Some(Token(TokenType::Operator(Operator::New), _)) => {}
        Some(token) => {
            return Err(ParserError(
                "Unexpected token in object creation".to_string(),
                token.1.clone(),
            ))
        }
        None => {
            return Err(ParserError(
                "Expected creation operator".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    let call = parse_call(token_stream, file)?;
    let type_ = parse_type(token_stream, file)?;

    return Ok(ObjectCreate {
        kind: type_,
        args: call,
    });
}
