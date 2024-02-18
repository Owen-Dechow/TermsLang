use crate::{
    errors::SyntaxError,
    lexer::tokens::{Operator, Token, TokenType},
};

use super::{
    parse_object::{parse_object_create, parse_object_peekable_callable},
    Object, ObjectCreate, TokenStream,
};

#[derive(Debug, Clone)]
pub enum OperandExpression {
    Unary {
        operand: Token,
        val: Box<OperandExpression>,
    },
    Binary {
        operand: Token,
        left: Box<OperandExpression>,
        right: Box<OperandExpression>,
    },
    Literal(Token),
    Object(Object),
    Create(ObjectCreate),
}

#[derive(Debug, Clone)]
enum OperandComponent {
    Object(Object),
    Literal(Token),
    Operand(Token),
    Create(ObjectCreate),
}

fn get_precedent_map() -> Vec<Vec<Operator>> {
    vec![
        vec![Operator::And, Operator::Or],
        vec![
            Operator::Equal,
            Operator::Greater,
            Operator::Less,
            Operator::GreaterOrEqual,
            Operator::LessOrEqual,
            Operator::NotEqual,
        ],
        vec![Operator::Add, Operator::Subtract],
        vec![Operator::Multiply, Operator::Divide, Operator::Modulo],
        vec![Operator::Exponent],
        vec![Operator::Not], // Uninary only
    ]
}

fn parse_slice(
    mut slice: Vec<OperandComponent>,
    president_map: &Vec<Vec<Operator>>,
) -> Result<OperandExpression, SyntaxError> {
    let mut paren_depth = 0;

    // Remove unnecessary parentheses: (1 + 1) -> 1 + 1
    loop {
        let len = slice.len();
        if let OperandComponent::Operand(Token(TokenType::Operator(Operator::OpenParen), ..)) =
            slice[0]
        {
            if let OperandComponent::Operand(Token(TokenType::Operator(Operator::CloseParen), ..)) =
                slice[len - 1]
            {
                slice.remove(0);
                slice.pop();
                continue;
            }
        }

        break;
    }

    // Check if value
    if slice.len() == 1 {
        match &slice[0] {
            OperandComponent::Literal(value) => {
                return Ok(OperandExpression::Literal(value.clone()))
            }
            OperandComponent::Object(value) => return Ok(OperandExpression::Object(value.clone())),
            OperandComponent::Create(value) => return Ok(OperandExpression::Create(value.clone())),
            OperandComponent::Operand(token) => {
                return Err(SyntaxError(
                    "Unexpected operator where value should be found".to_string(),
                    Some(token.1.clone()),
                ))
            }
        }
    }

    // Loop over to split on operators
    for (president_idx, president_layer) in president_map.into_iter().enumerate() {
        for (operand_component_idx, operand_component) in slice.clone().into_iter().enumerate() {
            if let OperandComponent::Operand(Token(TokenType::Operator(Operator::OpenParen), ..)) =
                operand_component
            {
                paren_depth += 1;
                continue;
            }

            if let OperandComponent::Operand(Token(
                TokenType::Operator(Operator::CloseParen),
                pos,
            )) = operand_component
            {
                paren_depth -= 1;

                // check for extra closing )
                if paren_depth <= -1 {
                    return Err(SyntaxError(
                        "Unmatched closing operand block found".to_string(),
                        Some(pos.clone()),
                    ));
                }

                continue;
            }

            if let OperandComponent::Literal(..) = operand_component {
                continue;
            }

            if paren_depth == 0 {
                if let OperandComponent::Operand(ref operand_token) = operand_component {
                    if let Token(TokenType::Operator(operator), ..) = operand_token {
                        if president_layer.contains(&operator) {
                            if president_idx == president_map.len() - 1 {
                                let slice = match slice.get(operand_component_idx + 1..) {
                                    Some(slice) => slice,
                                    None => {
                                        return Err(SyntaxError(
                                            "Expected value right of uniary operator".to_string(),
                                            Some(operand_token.1.clone()),
                                        ))
                                    }
                                };

                                if slice.len() == 0 {
                                    return Err(SyntaxError(
                                        "Expected value right of uniary operator".to_string(),
                                        Some(operand_token.1.clone()),
                                    ));
                                }

                                return Ok(OperandExpression::Unary {
                                    operand: operand_token.clone(),
                                    val: Box::new(parse_slice(
                                        slice.into_iter().cloned().collect(),
                                        president_map,
                                    )?),
                                });
                            }

                            let slice_l = match slice.get(..operand_component_idx) {
                                Some(slice_l) => slice_l,
                                None => {
                                    return Err(SyntaxError(
                                        "Expected value left of binary operator".to_string(),
                                        Some(operand_token.1.clone()),
                                    ))
                                }
                            };

                            let slice_r = match slice.get(operand_component_idx + 1..) {
                                Some(slice_r) => slice_r,
                                None => {
                                    return Err(SyntaxError(
                                        "Expected value right of binary operator".to_string(),
                                        Some(operand_token.1.clone()),
                                    ))
                                }
                            };

                            if slice_r.len() == 0 {
                                return Err(SyntaxError(
                                    "Expected value right of binary operator".to_string(),
                                    Some(operand_token.1.clone()),
                                ));
                            }

                            if slice_l.len() == 0 {
                                return Err(SyntaxError(
                                    "Expected value left of binary operator".to_string(),
                                    Some(operand_token.1.clone()),
                                ));
                            }

                            return Ok(OperandExpression::Binary {
                                operand: operand_token.clone(),
                                left: Box::new(parse_slice(
                                    slice_l.into_iter().cloned().collect(),
                                    president_map,
                                )?),
                                right: Box::new(parse_slice(
                                    slice_r.into_iter().cloned().collect(),
                                    president_map,
                                )?),
                            });
                        }
                    }
                }
            }
        }
    }

    return Err(SyntaxError(
        "Operand parse falls through".to_string(),
        match slice.last() {
            Some(OperandComponent::Operand(token)) => Some(token.1.clone()),
            _ => None,
        },
    ));
}

pub fn parse_operand_block(
    token_stream: &mut TokenStream,
    terminating_tokens: Vec<TokenType>,
) -> Result<OperandExpression, SyntaxError> {
    let operand_list = {
        let mut operand_list = Vec::<OperandComponent>::new();

        loop {
            let token = match token_stream.advance() {
                Some(token) => token,
                None => {
                    return Err(SyntaxError(
                        "Expected end of operand block".to_string(),
                        None,
                    ))
                }
            };

            // Check for terminating token
            let mut break_loop = false;
            for terminating_token in &terminating_tokens {
                if terminating_token == &token.0 {
                    break_loop = true;
                    break;
                }
            }
            if break_loop {
                break;
            }

            let translated_token = match token.0 {
                TokenType::Int(..) => OperandComponent::Literal(token.clone()),
                TokenType::Float(..) => OperandComponent::Literal(token.clone()),
                TokenType::String(..) => OperandComponent::Literal(token.clone()),
                TokenType::Identity(..) => {
                    token_stream.back();
                    OperandComponent::Object(parse_object_peekable_callable(token_stream)?)
                }
                TokenType::Operator(Operator::New) => {
                    token_stream.back();
                    OperandComponent::Create(parse_object_create(token_stream)?)
                }
                TokenType::Operator(..) => OperandComponent::Operand(token.clone()),
                _ => {
                    return Err(SyntaxError(
                        "Unexpected token in operand block".to_string(),
                        Some(token.1.clone()),
                    ))
                }
            };

            operand_list.push(translated_token);
        }

        operand_list
    };

    return parse_slice(operand_list, &get_precedent_map());
}
