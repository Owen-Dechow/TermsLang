use std::collections::HashMap;

use rand::random;

use crate::{
    errors::{FileLocation, RuntimeError},
    lexer::syntax::PROGRAM_ENTRY,
    parser::{Function, ObjectType, Program, Struct, TermBlock, Type},
};

#[derive(Debug)]
enum RootType {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Null,
}

#[derive(Debug)]
enum DataObject {
    StructObject {},
    RootObject(RootType),
}

#[derive(Debug)]
enum StructDef {
    User(HashMap<String, u32>),
    Root(RootType),
}

#[derive(Debug, Clone)]
struct FuncDef {
    returntype: u32,
    args: Vec<(String, u32)>,
    block: TermBlock,
}
impl FuncDef {
    fn call(&self, args: Vec<u32>, gc: &mut GarbageCollector) -> Result<(), RuntimeError> {
        println!("RUNNING FUNCTION");
        Ok(())
    }
}

#[derive(Debug)]
enum Data {
    StructDef(StructDef),
    FuncDef(FuncDef),
    DataObject(DataObject),
}

#[derive(Debug)]
struct DataCase {
    ref_count: u32,
    data: Data,
}

#[derive(Debug)]
struct VariableRegistry<'a> {
    vars: HashMap<String, u32>,
    parent: Option<&'a VariableRegistry<'a>>,
}
impl<'a> VariableRegistry<'a> {
    fn resolve_string(&self, string: String) -> Result<u32, RuntimeError> {
        match self.vars.get(&string) {
            Some(resolved) => Ok(*resolved),
            None => match self.parent {
                Some(parent) => parent.resolve_string(string),
                None => Err(RuntimeError(
                    format!("{} is not defined", string),
                    crate::errors::FileLocation::None,
                )),
            },
        }
    }
}

#[derive(Debug)]
struct GarbageCollector<'a> {
    variables: VariableRegistry<'a>,
    data: HashMap<u32, DataCase>,
}

impl<'a> GarbageCollector<'a> {
    fn new() -> GarbageCollector<'a> {
        let mut gc = GarbageCollector {
            variables: VariableRegistry {
                vars: HashMap::new(),
                parent: None,
            },
            data: HashMap::new(),
        };

        gc.add_root_type("int", RootType::Int(0));
        gc.add_root_type("bool", RootType::Bool(true));
        gc.add_root_type("float", RootType::Float(0.0));
        gc.add_root_type("str", RootType::String(String::new()));
        gc.add_root_type("null", RootType::Null);

        return gc;
    }

    fn add_root_type(&mut self, name: &str, root_type: RootType) {
        let key = random();
        let data_case = DataCase {
            ref_count: 1,
            data: Data::StructDef(StructDef::Root(root_type)),
        };
        self.data.insert(key, data_case);
        self.variables.vars.insert(name.to_string(), key);
    }

    fn add_var_from_data(&mut self, var: String, data: Data) {
        let key = random();
        let data_case = DataCase { ref_count: 1, data };
        self.data.insert(key, data_case);
        self.variables.vars.insert(var, key);
    }

    fn add_struct(&mut self, _struct: Struct) -> Result<(), RuntimeError> {
        let mut properties = HashMap::new();
        for property in _struct.properties {
            properties.insert(property.identity, self.resolve_type(property.argtype)?);
        }

        let struct_def = Data::StructDef(StructDef::User(properties));
        self.add_var_from_data(_struct.name, struct_def);

        return Ok(());
    }

    fn add_function(&mut self, func: Function) -> Result<(), RuntimeError> {
        let returntype = self.resolve_type(func.returntype)?;
        let args = {
            let mut args = Vec::new();
            for arg in func.args {
                args.push((arg.identity, self.resolve_type(arg.argtype)?));
            }
            args
        };
        let block = func.block;

        let func_def = Data::FuncDef(FuncDef {
            returntype,
            args,
            block,
        });
        self.add_var_from_data(func.name, func_def);

        return Ok(());
    }

    fn resolve_string(&self, string: String) -> Result<u32, RuntimeError> {
        self.variables.resolve_string(string)
    }

    fn resolve_type(&self, _type: Type) -> Result<u32, RuntimeError> {
        match _type {
            Type::Array(_) => todo!(),
            Type::Object { object } => match object.kind {
                ObjectType::Identity(id) => match object.sub {
                    Some(_) => Err(RuntimeError(
                        "Expected type".to_string(),
                        FileLocation::None,
                    )),
                    None => Ok(self.resolve_string(id)?),
                },
                _ => Err(RuntimeError(
                    "Expected type".to_string(),
                    FileLocation::None,
                )),
            },
        }
    }
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let mut gc = GarbageCollector::new();

    for _struct in program.structs {
        gc.add_struct(_struct)?;
    }

    for func in program.functions {
        gc.add_function(func)?;
    }

    match gc.variables.vars.get(PROGRAM_ENTRY) {
        Some(main_id) => match &gc.data[main_id].data {
            Data::FuncDef(func) => {
                let func = func.clone();
                func.call(Vec::new(), &mut gc)
            }
            _ => Err(RuntimeError(
                format!("{} must be a function.", PROGRAM_ENTRY),
                FileLocation::None,
            )),
        },
        None => Err(RuntimeError(
            format!("{} function not found.", PROGRAM_ENTRY),
            FileLocation::None,
        )),
    }
}
