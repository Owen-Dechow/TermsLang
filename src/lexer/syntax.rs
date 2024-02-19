use std::collections::HashMap;

use super::tokens::{KeyWord, Operator, StringInterpolator};

pub const WHITE_SPACE: &str = " \n\t";
pub const DECIMAL: char = '.';
pub const VARIABLE_ALLOWED_EXTRA_CHARS: &str = "@_";
pub const COMMENT: char = '#';
pub const NEW_LINE: char = '\n';
pub const LINE_TERMINATOR: char = ';';
pub const STRING_QUOTES: &str = "\"'`";
pub const FORMAT_STRING_GATES: (char, char) = ('{', '}');
pub const IGNORED_IN_NUMBERS: &str = "_";

pub const IDENTITY_PREFIX: &str = "t_";
pub const IDENTITY_PREFIX_FREE: [&str; 7] =
    ["null", "int", "str", "bool", "true", "false", "float"];

pub struct SyntaxMap<'a> {
    pub operators: HashMap<&'a str, Operator>,
    pub keywords: HashMap<&'a str, KeyWord>,
    pub string_interpolators: HashMap<&'a str, StringInterpolator>,
}

pub fn get_syntax_map() -> SyntaxMap<'static> {
    return SyntaxMap {
        operators: HashMap::<&str, Operator>::from([
            ("+", Operator::Add),
            ("-", Operator::Subtract),
            ("*", Operator::Multiply),
            ("/", Operator::Divide),
            ("%", Operator::Modulo),
            ("^", Operator::Exponent),
            ("(", Operator::OpenParen),
            (")", Operator::CloseParen),
            ("{", Operator::OpenBlock),
            ("}", Operator::CloseBlock),
            ("[", Operator::OpenBracket),
            ("]", Operator::CloseBracket),
            ("=", Operator::Set),
            ("+=", Operator::SetAdd),
            ("-=", Operator::SetSubtract),
            ("*=", Operator::SetMultiply),
            ("/=", Operator::SetDivide),
            ("%=", Operator::SetModulo),
            ("^=", Operator::SetExponent),
            ("==", Operator::Equal),
            (">", Operator::Greater),
            ("<", Operator::Less),
            (">=", Operator::GreaterOrEqual),
            ("<=", Operator::LessOrEqual),
            ("!=", Operator::NotEqual),
            ("!", Operator::Not),
            ("&&", Operator::And),
            ("||", Operator::Or),
            (".", Operator::Dot),
            (":", Operator::Colon),
            (",", Operator::Comma),
            ("$", Operator::New),
        ]),
        keywords: HashMap::<&str, KeyWord>::from([
            ("print", KeyWord::Print),
            ("println", KeyWord::PrintLn),
            ("readln", KeyWord::ReadLn),
            ("class", KeyWord::Class),
            ("if", KeyWord::If),
            ("else", KeyWord::Else),
            ("func", KeyWord::Func),
            ("let", KeyWord::Var),
            ("updt", KeyWord::UpdateVar),
            ("cll", KeyWord::Call),
            ("loop", KeyWord::Loop),
            ("break", KeyWord::Break),
            ("continue", KeyWord::Continue),
            ("return", KeyWord::Return),
        ]),
        string_interpolators: HashMap::<&str, StringInterpolator>::from([
            ("f", StringInterpolator::Interpolated),
            ("r", StringInterpolator::Raw),
        ]),
    };
}
