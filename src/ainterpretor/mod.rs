use std::collections::HashMap;

use crate::{
    active_parser::{AFunc, AProgram, AStruct},
    errors::RuntimeError,
};

struct GlobalData<'a> {
    structs: HashMap<&'a String, &'a AStruct>,
    functions: HashMap<&'a String, &'a AFunc>,
}
impl<'a> GlobalData<'a> {
    fn new() -> Self {
        Self {
            structs: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn add_struct(&mut self, _struct: &'a AStruct) {
        self.structs.insert(&_struct.name, _struct);
    }

    fn add_function(&mut self, func: &'a AFunc) {
        self.functions.insert(&func.name, func);
    }
}

struct DataScope<'a> {
    parent: Option<&'a DataScope<'a>>,
}
impl<'a> DataScope<'a> {
    fn new() -> Self {
        Self { parent: None }
    }

    fn from_parent(parent: &'a DataScope) -> Self {
        Self {
            parent: Some(parent),
        }
    }
}

fn interpret_function(func: &AFunc, parent_ds: &DataScope, gd: &GlobalData) {}

pub fn interpret(program: AProgram) -> Result<(), RuntimeError> {
    let mut gd = GlobalData::new();

    for _struct in &program.structs {
        gd.add_struct(_struct);
    }

    for func in &program.functions {
        gd.add_function(func);
    }

    return Ok(());
}
