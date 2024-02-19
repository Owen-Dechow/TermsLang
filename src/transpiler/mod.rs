use std::fs;

use crate::{
    errors::TranspilerError,
    lexer::{
        syntax::IDENTITY_PREFIX,
        tokens::{Operator, Token, TokenType},
    },
    parser::{
        parse_operand_block::OperandExpression, Call, Object, ObjectCreate, ObjectType, Term, Type,
        VarSigniture,
    },
};

fn translate_type_identity(id: &str) -> String {
    match id {
        "int" => "i32",
        "str" => "String",
        "float" => "f32",
        _ => id,
    }
    .to_string()
}

fn transpile_objectcreate(create: &ObjectCreate) -> Result<String, TranspilerError> {
    match &create.kind {
        Type::Array(type_) => return Ok(format!("Vec::<{}>::new()", transpile_type(&type_)?)),
        Type::Object { .. } => {
            return Ok(format!(
                "{}::new{}",
                transpile_type(&create.kind)?,
                transpile_call(&create.args)?
            ));
        }
    }
}

fn transpile_main_function(block: &Term) -> Result<String, TranspilerError> {
    Ok(format!(
        "fn main() {{ let {}args: Vec<String> = std::env::args().collect();{} }}",
        IDENTITY_PREFIX,
        transpile_term(block)?
    ))
}

fn transpile_call(call: &Call) -> Result<String, TranspilerError> {
    let mut result = "<".to_string();

    for typearg in &call.typeargs {
        result += &transpile_type(&typearg)?;
        result += ","
    }
    result += ">(";

    for arg in &call.args {
        result += &transpile_operand_expression(&arg)?;
        result += ","
    }
    result += ")";

    return Ok(result);
}

fn transpile_token_literal(token: &Token) -> Result<String, TranspilerError> {
    return Ok(match &token.0 {
        TokenType::Int(int) => int.to_string(),
        TokenType::Float(float) => float.to_string(),
        TokenType::String(string) => format!(r#" "{}".to_string()) "#, string),
        _ => {
            return Err(TranspilerError(
                "Should not find any token that cannot be read as literal in this location"
                    .to_string(),
                Some(token.1.clone()),
            ))
        }
    });
}

fn transpile_object(object: &Object) -> Result<String, TranspilerError> {
    let mut result = String::new();
    match &object.kind {
        ObjectType::Identity(id) => match &id.0 {
            TokenType::Identity(id) => {
                result += &translate_type_identity(id).replace("@", "__at__")
            }
            _ => {
                return Err(TranspilerError(
                    "Should not find any token other than identity in this location".to_string(),
                    Some(id.1.clone()),
                ))
            }
        },
        ObjectType::Call(call) => {
            result += &format!("::{}", &transpile_call(&call)?);
        }
        ObjectType::Index(idx) => {
            result += &format!("[{}]", transpile_operand_expression(&idx)?);
        }
    }

    match &object.sub {
        Some(sub) => match sub.kind {
            ObjectType::Identity(..) => {
                return Ok(format!("{}.{}", result, transpile_object(&sub)?))
            }
            _ => return Ok(format!("{}{}", result, transpile_object(&sub)?)),
        },
        None => {
            return Ok(result);
        }
    }
}

fn transpile_type(type_: &Type) -> Result<String, TranspilerError> {
    match type_ {
        Type::Array(type_) => {
            return Ok(format!("Vec<{}>", transpile_type(type_)?));
        }
        Type::Object {
            object,
            associated_types,
        } => {
            let mut result = transpile_object(&object)?;

            if associated_types.len() > 0 {
                result += "<";
                for associated_type in associated_types {
                    result += &transpile_type(&associated_type)?;
                    result += ","
                }
                result += ">";
            };

            return Ok(result);
        }
    }
}

fn transpile_var_signiture(var_sig: &VarSigniture) -> Result<String, TranspilerError> {
    return Ok(format!(
        "{}: {}",
        transpile_object(&var_sig.identity)?,
        transpile_type(&var_sig.argtype)?
    ));
}

fn transpile_operand_expression(
    operand_expression: &OperandExpression,
) -> Result<String, TranspilerError> {
    return Ok(match operand_expression {
        OperandExpression::Unary { operand, val } => {
            format!(
                "{}({})",
                transpile_operator(match &operand.0 {
                    TokenType::Operator(operator) => &operator,
                    _ =>
                        return Err(TranspilerError(
                            "Should not find any token other than operator in this location"
                                .to_string(),
                            Some(operand.1.clone())
                        )),
                })?,
                transpile_operand_expression(val)?
            )
        }
        OperandExpression::Binary {
            operand,
            left,
            right,
        } => {
            format!(
                "({}) {} ({})",
                transpile_operand_expression(left)?,
                transpile_operator(match &operand.0 {
                    TokenType::Operator(operator) => &operator,
                    _ =>
                        return Err(TranspilerError(
                            "Should not find any token other than operator in this location"
                                .to_string(),
                            Some(operand.1.clone())
                        )),
                })?,
                transpile_operand_expression(right)?
            )
        }
        OperandExpression::Literal(literal) => transpile_token_literal(literal)?,
        OperandExpression::Object(object) => transpile_object(object)?,
        OperandExpression::Create(create) => transpile_objectcreate(create)?,
    });
}

fn transpile_operator(operator: &Operator) -> Result<String, TranspilerError> {
    return Ok(match operator {
        Operator::Add => "+",
        Operator::Subtract => "-",
        Operator::Multiply => "*",
        Operator::Divide => "/",
        Operator::Modulo => "%",

        Operator::Set => "=",
        Operator::SetAdd => "+=",
        Operator::SetSubtract => "-=",
        Operator::SetMultiply => "*=",
        Operator::SetDivide => "/=",
        Operator::SetModulo => "%=",

        Operator::Equal => "==",
        Operator::Greater => ">",
        Operator::Less => "<",
        Operator::GreaterOrEqual => ">=",
        Operator::LessOrEqual => "<=",
        Operator::NotEqual => "!=",
        Operator::Not => "!",
        Operator::And => "&&",
        Operator::Or => "||",
        _ => {
            return Err(TranspilerError(
                format!("Operator '{}' should never be transpiled", operator),
                None,
            ))
        }
    }
    .to_string());
}

fn transpile_term(term: &Term) -> Result<String, TranspilerError> {
    let mut result = String::new();

    match term {
        Term::Block { terms } => {
            result += "{\n";
            for term in terms {
                result += &transpile_term(term)?;
            }
            result += "\n}";
        }
        Term::Func {
            name,
            returntype,
            typeargs,
            args,
            block,
        } => {
            let name = transpile_object(name)?;
            if name == format!("{}{}", IDENTITY_PREFIX, "main") {
                result += &transpile_main_function(block)?;
            } else {
                result += &format!("fn {name} <");

                for typearg in typeargs {
                    result += &transpile_object(typearg)?;
                    result += ","
                }
                result += ">(";

                for arg in args {
                    result += &transpile_var_signiture(arg)?;
                    result += ","
                }
                result += ") -> ";

                result += &transpile_type(returntype)?;
                result += &transpile_term(block)?;
            }
        }
        Term::Print { ln, operand_block } => {
            let expression = transpile_operand_expression(operand_block)?;
            if *ln {
                result = format!(r#"println!("{{:?}}", {});"#, expression)
            } else {
                result = format!(r#"print!("{{:?}}", {});"#, expression)
            }
        }
        Term::DeclareVar {
            name,
            vartype,
            value,
        } => {
            result = format!(
                "let mut {}: {} = {};",
                transpile_object(name)?,
                transpile_type(vartype)?,
                transpile_operand_expression(value)?
            )
        }
        Term::Return { value } => {
            result = format!("return {};", transpile_operand_expression(value)?)
        }
        Term::UpdateVar {
            var,
            set_operator,
            value,
        } => {
            result = format!(
                "{} {} {};",
                transpile_object(var)?,
                transpile_operator(set_operator)?,
                transpile_operand_expression(value)?
            )
        }
        Term::If {
            conditional,
            block,
            else_block,
        } => {
            result = format!(
                "if {} {} else {}",
                transpile_operand_expression(conditional)?,
                transpile_term(block)?,
                transpile_term(else_block)?
            )
        }
        Term::Loop {
            counter,
            conditional,
            block,
        } => {
            let i = transpile_object(counter)?;

            result = format!(
                "{{ let mut {i} = 0; while {} {{ {i} += 1; let mut {i} = {i} - 1; {} }} }}",
                transpile_operand_expression(conditional)?,
                transpile_term(block)?
            )
        }
        Term::ReadLn { var } => {
            result = format!(
                "std::io::stdin().read_line(&mut {}.0).unwrap();",
                transpile_object(var)?
            )
        }
        Term::Break => result = "break;".to_string(),
        Term::Continue => result = "continue;".to_string(),
        Term::Call { value } => result = format!("{};", transpile_operand_expression(value)?),
    }

    return Ok(result);
}

pub fn transpile(block: &Term) -> Result<String, TranspilerError> {
    let mut result = fs::read_to_string("internals/prelude.rs").unwrap();

    if let Term::Block { terms } = block {
        for term in terms {
            result += &transpile_term(term)?;
        }
    }

    result = result.replace(";", ";\n");

    return Ok(result);
}
