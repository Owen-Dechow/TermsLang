use crate::lexer::tokens::Operator;

pub const PROGRAM_ENTRY: &str = "@main";
pub const NULL_STRING: &str = "null";

pub const STRUCT_SELF: &str = "@this";

pub const ADD_FUNC: &str = "@add";
pub const SUBTRACT_FUNC: &str = "@subtract";
pub const MULTIPLY_FUNC: &str = "@multiply";
pub const DIVIDE_FUNC: &str = "@divide";
pub const MODULO_FUNC: &str = "@modulo";
pub const EXPONENT_FUNC: &str = "@exponent";
pub const EQUAL_FUNC: &str = "@equal";
pub const GREATER_FUNC: &str = "@greater";
pub const GREATER_OR_EQUAL_FUNC: &str = "@greaterorequal";
pub const LESS_FUNC: &str = "@less";
pub const LESS_OR_EQUAL_FUNC: &str = "@lessorequal";
pub const NOT_FUNC: &str = "@not";
pub const AND_FUNC: &str = "@and";
pub const OR_FUNC: &str = "@or";

pub const TO_STRING_FUNC: &str = "@str";
pub const TO_INT_FUNC: &str = "@int";
pub const TO_FLOAT_FUNC: &str = "@float";

pub const NEW_FUNC: &str = "@new";

pub const APPEND_FUNC: &str = "@append";
pub const REMOVE_FUNC: &str = "@remove";
pub const LEN_FUNC: &str = "@len";
pub const INDEX_FUNC: &str = "@index";

pub const ARRAY_METHODS: [&str; 4] = [APPEND_FUNC, REMOVE_FUNC, LEN_FUNC, INDEX_FUNC];

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
        Operator::GreaterOrEqual => GREATER_OR_EQUAL_FUNC,
        Operator::Less => LESS_FUNC,
        Operator::LessOrEqual => LESS_OR_EQUAL_FUNC,
        Operator::Not => NOT_FUNC,
        Operator::Or => OR_FUNC,
        Operator::And => AND_FUNC,
        _ => todo!(),
    }
    .to_string()
}
