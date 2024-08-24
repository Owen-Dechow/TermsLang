use std::path::PathBuf;

use crate::{
    errors::{FileLocation, ParserError},
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{parse_object::parse_object_peekable, TokenStream, Type, VarSigniture};

// Get types within <>
pub fn get_associated_types(
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<Vec<Type>, ParserError> {
    // Create the type list
    let mut associated_types = Vec::<Type>::new();

    loop {
        // Add clause to check for empty type list: <>
        if let Some(Token(TokenType::Operator(Operator::Greater), _)) = token_stream.advance() {
            break;
        } else {
            token_stream.back();
        }

        // Parse type
        let associated_type = parse_type(token_stream, file)?;

        // Add valid type to list
        associated_types.push(associated_type);

        // Check for , else if > break loop else return error
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Greater), _)) => break,
            Some(Token(TokenType::Operator(Operator::Comma), _)) => continue,
            Some(token) => {
                return Err(ParserError(
                    "Unexpected token in associated type list".to_string(),
                    token.1.clone(),
                ))
            }
            None => {
                return Err(ParserError(
                    "Expected associated type list close".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        }
    }

    // return associated types
    return Ok(associated_types);
}

// Parse a type identifier
pub fn parse_type(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Type, ParserError> {
    // Get the typename token
    let typename = parse_object_peekable(token_stream, file)?;

    let mut _type = Type::Object { object: typename };

    // Wrap matrix
    while let Some(Token(TokenType::Operator(Operator::OpenBracket), _)) = token_stream.advance() {
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::CloseBracket), _)) => {
                _type = Type::Array(Box::new(_type));
            }
            Some(token) => {
                return Err(ParserError(
                    "Unexpected token instead of closing bracket".to_string(),
                    token.1.clone(),
                ))
            }
            None => {
                return Err(ParserError(
                    "Expected closing bracket".to_string(),
                    FileLocation::End { file: file.clone() },
                ))
            }
        }
    }

    token_stream.back();
    return Ok(_type);
}

// Gen variable signiture: type<>[] name
pub fn parse_var_sig(
    token_stream: &mut TokenStream,
    file: &PathBuf,
) -> Result<VarSigniture, ParserError> {
    // Get the type of the argument
    let argtype = parse_type(token_stream, file)?;

    // Get the name of the argument
    let name = match token_stream.advance() {
        Some(op) => match &op.0 {
            TokenType::Identity(id) => id.to_owned(),
            _ => {
                return Err(ParserError(
                    "Unexpected token in place of varible name".to_string(),
                    op.1.clone(),
                ))
            }
        },
        None => {
            return Err(ParserError(
                "Expected variable name".to_string(),
                FileLocation::End { file: file.clone() },
            ))
        }
    };

    // Return the new variable signiture
    return Ok(VarSigniture {
        identity: name,
        argtype,
    });
}
