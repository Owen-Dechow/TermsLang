use crate::{
    errors::SyntaxError,
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{
    parse_object::{parse_object, parse_object_peekable},
    Object, TokenStream, Type, VarSigniture,
};

// Get types within <>
pub fn get_associated_types(token_stream: &mut TokenStream) -> Result<Vec<Type>, SyntaxError> {
    // Create the type list
    let mut associated_types = Vec::<Type>::new();

    loop {
        // Add clause to check for empty type list: <>
        if let Some(Token(TokenType::Operator(Operator::Greater), ..)) = token_stream.advance() {
            break;
        } else {
            token_stream.back();
        }

        // Parse type
        let associated_type = parse_type(token_stream)?;

        // Add valid type to list
        associated_types.push(associated_type);

        // Check for , else if > break loop else return error
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Greater), ..)) => break,
            Some(Token(TokenType::Operator(Operator::Comma), ..)) => continue,
            Some(token) => {
                return Err(SyntaxError(
                    "Unexpected token in associated type list".to_string(),
                    Some(token.1.clone()),
                ))
            }
            None => {
                return Err(SyntaxError(
                    "Expected associated type list close".to_string(),
                    None,
                ))
            }
        }
    }

    // return associated types
    return Ok(associated_types);
}

// Get type args: <T, T2>
pub fn get_type_args(token_stream: &mut TokenStream) -> Result<Vec<Object>, SyntaxError> {
    // Create type args list
    let mut type_args = Vec::<Object>::new();

    // Get args
    loop {
        if let Some(Token(TokenType::Operator(Operator::Greater), ..)) = token_stream.advance() {
            break;
        }

        token_stream.back();
        type_args.push(parse_object(token_stream)?);

        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::Comma), ..)) => {}
            _ => {
                token_stream.back();
            }
        };
    }

    // Return list
    return Ok(type_args);
}

// Parse a type identifier
pub fn parse_type(token_stream: &mut TokenStream) -> Result<Type, SyntaxError> {
    // Get the typename token
    let typename = parse_object_peekable(token_stream)?;

    // collect the next token if none exists return type
    let next = match token_stream.advance() {
        Some(next) => next,
        None => {
            return Ok(Type::Object {
                object: typename,
                associated_types: Vec::<Type>::new(),
            });
        }
    };

    // check for associated types: <>
    let associated_types = {
        if let Token(TokenType::Operator(Operator::Less), ..) = next {
            get_associated_types(token_stream)?
        } else {
            token_stream.back();
            Vec::<Type>::new()
        }
    };

    let mut _type = Type::Object {
        object: typename,
        associated_types,
    };

    // Wrap matrix
    while let Some(Token(TokenType::Operator(Operator::OpenBracket), ..)) = token_stream.advance() {
        match token_stream.advance() {
            Some(Token(TokenType::Operator(Operator::CloseBracket), ..)) => {
                _type = Type::Array(Box::new(_type));
            }
            Some(token) => {
                return Err(SyntaxError(
                    "Unexpected token instead of closing bracket".to_string(),
                    Some(token.1.clone()),
                ))
            }
            None => return Err(SyntaxError("Expected closing bracket".to_string(), None)),
        }
    }

    token_stream.back();
    return Ok(_type);
}

// Gen variable signiture: type<>[] name
pub fn get_var_sig(token_stream: &mut TokenStream) -> Result<VarSigniture, SyntaxError> {
    // Get the type of the argument
    let argtype = parse_type(token_stream)?;

    // Get the name of the argument
    let name = parse_object(token_stream)?;

    // Return the new variable signiture
    return Ok(VarSigniture {
        identity: name,
        argtype,
    });
}
