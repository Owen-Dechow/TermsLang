use std::path::PathBuf;

use crate::{
    errors::{FileLocation, ParserError},
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{parse_object::parse_object_peekable, TokenStream, Type, VarSigniture};

// Parse a type identifier
pub fn parse_type(token_stream: &mut TokenStream, file: &PathBuf) -> Result<Type, ParserError> {
    // Get the typename token
    let typename = parse_object_peekable(token_stream, file)?;
    let location = typename.loc.clone();
    let mut _type = Type::Object { object: typename };

    // Wrap matrix
    while let Some(Token(TokenType::Operator(Operator::OpenBracket), _)) = token_stream.advance() {
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::CloseBracket), _)) => {
                _type = Type::Array {
                    _type: Box::new(_type),
                    location: location.clone(),
                };
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
