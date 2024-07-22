use crate::lexer::tokens::Operator;

pub const PROGRAM_ENTRY: &str = "@main";
pub const NULL_STRING: &str = "null";

pub const ADD_FUNC: &str = "@add";
pub const SUBTRACT_FUNC: &str = "@subtract";
pub const MULTIPLY_FUNC: &str = "@multiply";
pub const DIVIDE_FUNC: &str = "@divide";
pub const MODULO_FUNC: &str = "@modulo";
pub const EXPONENT_FUNC: &str = "@exponent";
pub const EQUAL_FUNC: &str = "@equal";
pub const GREATER_FUNC: &str = "@greater";
pub const LESS_FUNC: &str = "@less";
pub const NOT_FUNC: &str = "@not";
pub const OR_FUNC: &str = "@or";

pub const TO_STRING_FUNC: &str = "@str";

pub const OVERRIDEABLE_METHODS: [&str; 1] = [TO_STRING_FUNC];

pub fn convert_operator(operator: &Operator) -> String {
    match operator {
        Operator::Add => ADD_FUNC,
        Operator::Subtract => SUBTRACT_FUNC,
        Operator::Multiply => MULTIPLY_FUNC,
        Operator::Divide => DIVIDE_FUNC,
        Operator::Modulo => MODULO_FUNC,
        Operator::Exponent => EXPONENT_FUNC,
        Operator::Equal => EQUAL_FUNC,
        Operator::Greater => GREATER_FUNC,
        Operator::Less => LESS_FUNC,
        Operator::Not => NOT_FUNC,
        Operator::And => ADD_FUNC,
        Operator::Or => OR_FUNC,
        _ => todo!(),
    }
    .to_string()
}
