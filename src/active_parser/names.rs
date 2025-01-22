pub const BOOL: &str = "bool";
pub const INT: &str = "int";
pub const FLOAT: &str = "float";
pub const NULL: &str = "null";
pub const STRING: &str = "str";

pub const F_BOOL: &str = "@bool";
pub const F_INT: &str = "@int";
pub const F_FLOAT: &str = "@float";
pub const F_STRING: &str = "@str";

pub const F_NEW: &str = "@new";
pub const F_ADD: &str = "@add";
pub const F_SUB: &str = "@sub";
pub const F_MULT: &str = "@mult";
pub const F_DIV: &str = "@div";
pub const F_MOD: &str = "@mod";
pub const F_EXP: &str = "@exp";
pub const F_EQ: &str = "@eq";
pub const F_GT: &str = "@gt";
pub const F_GTEQ: &str = "@gteq";
pub const F_LT: &str = "@lt";
pub const F_LTEQ: &str = "@lteq";
pub const F_NOT: &str = "@not";
pub const F_AND: &str = "@and";
pub const F_OR: &str = "@or";
pub const F_LEN: &str = "@len";
pub const F_INDEX: &str = "@idx";
pub const F_APPEND: &str = "@append";
pub const F_REMOVE: &str = "@remove";

pub const F_READLN: &str = "@readln";
pub const F_MAIN: &str = "@main";

pub const THIS: &str = "@this";

pub const PREFIX_PROTECTED_NAMES: [&str; 30] = [
    BOOL, INT, FLOAT, NULL, STRING, F_BOOL, F_INT, F_FLOAT, F_STRING, F_NEW, F_ADD, F_SUB, F_MULT,
    F_DIV, F_MOD, F_EXP, F_EQ, F_GT, F_GTEQ, F_LT, F_LTEQ, F_NOT, F_AND, F_OR, F_LEN, F_INDEX,
    F_APPEND, F_REMOVE, F_READLN, THIS,
];
