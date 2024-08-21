use crate::errors::FileLocation;
use std::fmt::{Debug, Display};

#[derive(PartialEq, Clone)]
pub struct Token(pub TokenType, pub FileLocation);
impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Token").field(&self.0).finish()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identity(String),
    Operator(Operator),
    KeyWord(KeyWord),
    Comment(String),
    Terminate,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TokenType::Int(int) => format!("Int:{int}"),
            TokenType::Float(float) => format!("Float:{float}"),
            TokenType::String(string) => format!("String:\"{string}\""),
            TokenType::Bool(b) => format!("Bool:\"{b}\""),
            TokenType::Identity(identity) => format!("Identity:{identity}"),
            TokenType::Operator(operator) => format!("Operator:{operator}"),
            TokenType::KeyWord(keyword) => format!("KewWord:{keyword}"),
            TokenType::Terminate => format!("Terminator"),
            TokenType::Comment(comment) => format!("String:\"{comment}\""),
        };

        return write!(f, "Token({name})");
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,

    OpenParen,
    CloseParen,
    OpenBlock,
    CloseBlock,
    OpenBracket,
    CloseBracket,

    Set,
    SetAdd,
    SetSubtract,
    SetMultiply,
    SetDivide,
    SetModulo,
    SetExponent,

    Equal,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
    Not,
    And,
    Or,

    Dot,
    Colon,
    Comma,

    New,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum KeyWord {
    Print,
    PrintLn,
    Struct,
    If,
    Else,
    Func,
    Var,
    Return,
    UpdateVar,
    Loop,
    Break,
    Continue,
    Call,
    Static,
    Import,
}
impl Display for KeyWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
