mod internal_funcs;

use std::{collections::HashMap, env, iter::zip};

use rand::random;

use crate::{
    errors::{FileLocation, RuntimeError},
    lexer::{
        syntax::PROGRAM_ENTRY,
        tokens::{Token, TokenType},
    },
    parser::{
        parse_operand_block::OperandExpression, Function, Object, ObjectCreate, ObjectType,
        Program, Struct, Term, TermBlock, Type,
    },
};

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
enum RootType {
    String,
    Int,
    Float,
    Bool,
    Null,
}

#[derive(Debug, Clone)]
enum RootObject {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
struct StructObject(HashMap<String, u32>);

#[derive(Debug, Clone)]
struct ArrayObject(Vec<u32>);

#[derive(Debug, Clone)]
enum DataObject {
    StructObject(StructObject),
    RootObject(RootObject),
    ArrayObject(ArrayObject),
}

#[derive(Debug, Clone)]
enum StructDef {
    User(Vec<(String, TypeResolve)>),
    Root(RootType),
}

#[derive(Debug, Clone)]
enum RootFunc {
    ReadLn,
}
impl RootFunc {
    fn call(
        &self,
        gc: &mut GarbageCollector,
        vr: &VariableRegistry,
    ) -> Result<ExitMethod, RuntimeError> {
        internal_funcs::readln(gc)
    }
}

#[derive(Debug, Clone)]
enum FuncBlock {
    User(TermBlock),
    Root(RootFunc),
}

#[derive(Debug, Clone)]
struct FuncDef {
    returntype: TypeResolve,
    args: Vec<(String, TypeResolve)>,
    block: FuncBlock,
}

impl FuncDef {
    fn call(&self, args: Vec<u32>, gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
        let mut vr = gc.new_function_scope();
        if args.len() != self.args.len() {
            return Err(RuntimeError(
                format!("Function call has invalid arguments."),
                FileLocation::None,
            ));
        }

        for (ref_id, (arg_name, _arg_type)) in zip(&args, &self.args) {
            let data_case = gc.data.get_mut(&ref_id).unwrap();
            data_case.ref_count += 1;
            vr.vars.insert(arg_name.clone(), *ref_id);
        }

        let result = match &self.block {
            FuncBlock::User(term_block) => run_term_block(term_block, gc, &vr)?,
            FuncBlock::Root(root) => root.call(gc, &vr)?,
        };

        vr.release(gc);

        return match result {
            ExitMethod::ImplicitNullReturn => Ok(result),
            ExitMethod::ExplicitReturn(_) => Ok(result),
            ExitMethod::LoopContinue => Err(RuntimeError(
                format!("Cannot continue from outside loop."),
                FileLocation::None,
            )),
            ExitMethod::LoopBreak => Err(RuntimeError(
                format!("Cannot break from outside loop."),
                FileLocation::None,
            )),
        };
    }
}

#[derive(Debug, Clone)]
enum TypeResolve {
    Array(Box<TypeResolve>),
    Standard(u32),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct VariableRegistry {
    vars: HashMap<String, u32>,
    parent: Option<Box<VariableRegistry>>,
}
impl VariableRegistry {
    fn resolve_string(&self, string: &String) -> Result<u32, RuntimeError> {
        match self.vars.get(string) {
            Some(resolved) => Ok(*resolved),
            None => match &self.parent {
                Some(parent) => parent.resolve_string(string),
                None => Err(RuntimeError(
                    format!("{} is not defined", string),
                    crate::errors::FileLocation::None,
                )),
            },
        }
    }

    fn release(self, gc: &mut GarbageCollector) {
        for (_var, ref_id) in self.vars {
            let data_case = gc.data.get_mut(&ref_id).unwrap();
            data_case.ref_count -= 1;
            if data_case.ref_count == 0 {
                gc.data.remove(&ref_id);
            }
        }
    }

    fn create_child(&self) -> VariableRegistry {
        VariableRegistry {
            vars: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    fn add_var(&mut self, name: &String, data_ref: u32) {
        self.vars.insert(name.clone(), data_ref);
    }
}

#[derive(Debug)]
struct GarbageCollector {
    root_var_registry: VariableRegistry,
    data: HashMap<u32, DataCase>,
    root_type_map: HashMap<RootType, u32>,
    command_line_args: u32,
}
impl GarbageCollector {
    fn new() -> GarbageCollector {
        let mut gc = GarbageCollector {
            root_var_registry: VariableRegistry {
                vars: HashMap::new(),
                parent: None,
            },
            data: HashMap::new(),
            root_type_map: HashMap::new(),
            command_line_args: random(),
        };

        gc.add_root_type("int", RootType::Int);
        gc.add_root_type("bool", RootType::Bool);
        gc.add_root_type("float", RootType::Float);
        gc.add_root_type("str", RootType::String);
        gc.add_root_type("null", RootType::Null);

        gc.add_root_func(
            "readln",
            RootFunc::ReadLn,
            Vec::new(),
            TypeResolve::Standard(gc.root_type_map[&RootType::String]),
        );

        let mut command_line_args = Vec::new();
        for arg in env::args() {
            let key = random();
            command_line_args.push(key);

            gc.data.insert(
                key,
                DataCase {
                    ref_count: 1,
                    data: Data::DataObject(DataObject::RootObject(RootObject::String(arg))),
                },
            );
        }

        gc.data.insert(
            gc.command_line_args,
            DataCase {
                ref_count: 0,
                data: Data::DataObject(DataObject::ArrayObject(ArrayObject(command_line_args))),
            },
        );

        return gc;
    }

    fn new_function_scope(&self) -> VariableRegistry {
        VariableRegistry {
            vars: HashMap::new(),
            parent: Some(Box::new(self.root_var_registry.clone())),
        }
    }

    fn add_root_type(&mut self, name: &str, root_type: RootType) {
        let key = random();
        let data_case = DataCase {
            ref_count: 1,
            data: Data::StructDef(StructDef::Root(root_type.clone())),
        };
        self.data.insert(key, data_case);
        self.root_var_registry.vars.insert(name.to_string(), key);
        self.root_type_map.insert(root_type, key);
    }

    fn add_root_func(
        &mut self,
        name: &str,
        func: RootFunc,
        args: Vec<(String, TypeResolve)>,
        returntype: TypeResolve,
    ) {
        let data = Data::FuncDef(FuncDef {
            returntype,
            args,
            block: FuncBlock::Root(func),
        });

        let key = random();
        self.data.insert(key, DataCase { ref_count: 1, data });
        self.root_var_registry.vars.insert(name.to_string(), key);
    }

    fn add_var_from_data(&mut self, var: String, data: Data) {
        let key = random();
        let data_case = DataCase { ref_count: 1, data };
        self.data.insert(key, data_case);
        self.root_var_registry.vars.insert(var, key);
    }

    fn add_struct(&mut self, _struct: Struct) -> Result<(), RuntimeError> {
        let mut properties = Vec::new();
        for property in _struct.properties {
            properties.push((property.identity, self.resolve_type(&property.argtype)?));
        }

        let struct_def = Data::StructDef(StructDef::User(properties));
        self.add_var_from_data(_struct.name, struct_def);

        return Ok(());
    }

    fn add_function(&mut self, func: Function) -> Result<(), RuntimeError> {
        let returntype = self.resolve_type(&func.returntype)?;
        let args = {
            let mut args = Vec::new();
            for arg in func.args {
                args.push((arg.identity, self.resolve_type(&arg.argtype)?));
            }
            args
        };
        let block = FuncBlock::User(func.block);

        let func_def = Data::FuncDef(FuncDef {
            returntype,
            args,
            block,
        });
        self.add_var_from_data(func.name, func_def);

        return Ok(());
    }

    fn resolve_type(&self, _type: &Type) -> Result<TypeResolve, RuntimeError> {
        match _type {
            Type::Array(_type) => Ok(TypeResolve::Array(Box::new(self.resolve_type(&*_type)?))),
            Type::Object { object } => match &object.kind {
                ObjectType::Identity(id) => match object.sub {
                    Some(_) => Err(RuntimeError(
                        "Expected type".to_string(),
                        FileLocation::None,
                    )),
                    None => Ok(TypeResolve::Standard(
                        self.root_var_registry.resolve_string(id)?,
                    )),
                },
                _ => Err(RuntimeError(
                    "Expected type".to_string(),
                    FileLocation::None,
                )),
            },
        }
    }

    fn resolve_object(
        &mut self,
        object: &Object,
        vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        match &object.kind {
            ObjectType::Identity(id) => {
                let key = vr.resolve_string(id);
                match &object.sub {
                    Some(sub) => self.resolve_sub_object(key?, &*sub, vr),
                    None => key,
                }
            }
            ObjectType::Call(_) => Err(RuntimeError(
                "Cannot directly resolve call".to_string(),
                FileLocation::None,
            )),
            ObjectType::Index(_) => Err(RuntimeError(
                "Cannot directly resolve index".to_string(),
                FileLocation::None,
            )),
        }
    }

    fn resolve_sub_object(
        &mut self,
        parent: u32,
        object: &Object,
        vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        match &object.kind {
            ObjectType::Identity(id) => match &self.data.get(&parent).unwrap().data {
                Data::StructDef(_) => Err(RuntimeError(
                    "Struct definition cannot be child".to_string(),
                    FileLocation::None,
                )),
                Data::FuncDef(_) => Err(RuntimeError(
                    "Func definition cannot be child".to_string(),
                    FileLocation::None,
                )),
                Data::DataObject(data) => match data {
                    DataObject::StructObject(struct_object) => match struct_object.0.get(id) {
                        Some(key) => match &object.sub {
                            Some(sub) => self.resolve_sub_object(*key, &*sub, vr),
                            None => Ok(*key),
                        },
                        None => Err(RuntimeError(
                            format!("No field {} on struct", id),
                            FileLocation::None,
                        )),
                    },
                    DataObject::RootObject(_) => Err(RuntimeError(
                        "Root types do not have fields".to_string(),
                        FileLocation::None,
                    )),
                    DataObject::ArrayObject(_) => Err(RuntimeError(
                        "Array types do not have fields".to_string(),
                        FileLocation::None,
                    )),
                },
            },
            ObjectType::Call(call) => {
                if let Data::FuncDef(func_def) = self.data.get(&parent).unwrap().data.clone() {
                    let mut args = Vec::new();
                    for arg in &call.args {
                        args.push(interpret_operand_expression(arg, self, vr)?);
                    }
                    match func_def.call(args, self)? {
                        ExitMethod::ImplicitNullReturn => {
                            let key = random();
                            self.data.insert(
                                key,
                                DataCase {
                                    ref_count: 0,
                                    data: Data::DataObject(DataObject::RootObject(
                                        RootObject::Null,
                                    )),
                                },
                            );

                            Ok(key)
                        }
                        ExitMethod::ExplicitReturn(result) => Ok(result),
                        _ => {
                            return Err(RuntimeError(
                                "Invalid return from function call".to_string(),
                                FileLocation::None,
                            ))
                        }
                    }
                } else {
                    return Err(RuntimeError(
                        "Cannot call non function type".to_string(),
                        FileLocation::None,
                    ));
                }
            }
            ObjectType::Index(_) => todo!(),
        }
    }

    fn id_to_string(&self, id: u32) -> String {
        match &self.data.get(&id).unwrap().data {
            Data::StructDef(_) => format!("<struct {id}>"),
            Data::FuncDef(_) => format!("<func {id}>"),
            Data::DataObject(object) => match object {
                DataObject::StructObject(_) => format!("<userobject {id}>"),
                DataObject::RootObject(root) => match root {
                    RootObject::String(string) => string.to_owned(),
                    RootObject::Int(int) => int.to_string(),
                    RootObject::Float(float) => float.to_string(),
                    RootObject::Bool(_bool) => _bool.to_string(),
                    RootObject::Null => format!("null"),
                },
                DataObject::ArrayObject(arr) => {
                    let mut result = String::from("[");
                    for element in &arr.0 {
                        result += &self.id_to_string(*element);
                    }

                    result
                }
            },
        }
    }

    fn id_to_bool(&self, id: u32) -> bool {
        true
    }

    fn add_literal(&mut self, literal: &Token) -> Result<u32, RuntimeError> {
        let root = match &literal.0 {
            TokenType::Int(int) => RootObject::Int(*int),
            TokenType::Float(float) => RootObject::Float(*float),
            TokenType::String(string) => RootObject::String(string.to_owned()),
            TokenType::Bool(_bool) => RootObject::Bool(*_bool),
            _ => {
                return Err(RuntimeError(
                    format!("Invalid token literal in operand block."),
                    literal.1.clone(),
                ))
            }
        };

        let key = random();
        self.data.insert(
            key,
            DataCase {
                ref_count: 0,
                data: Data::DataObject(DataObject::RootObject(root)),
            },
        );
        return Ok(key);
    }

    fn create_object(
        &mut self,
        create: &ObjectCreate,
        vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        let key = random();
        let _type = self.resolve_type(&create.kind)?;
        let data = match _type {
            TypeResolve::Array(arr) => {
                if create.args.args.len() > 0 {
                    return Err(RuntimeError(
                        "Cannot have args in array creation.".to_string(),
                        FileLocation::None,
                    ));
                }

                Data::DataObject(DataObject::ArrayObject(ArrayObject(Vec::new())))
            }
            TypeResolve::Standard(std) => {
                if let Data::StructDef(struct_def) = self.data.get(&std).unwrap().data.clone() {
                    match struct_def {
                        StructDef::Root(root_def) => {
                            return Err(RuntimeError(
                                "Cannot directly construct root type".to_string(),
                                FileLocation::None,
                            ));
                        }
                        StructDef::User(user_def) => {
                            if user_def.len() != create.args.args.len() {
                                return Err(RuntimeError(
                                    "Invalid arguments to struct creation".to_string(),
                                    FileLocation::None,
                                ));
                            };

                            let mut fields = HashMap::new();
                            for (arg, (field_name, _)) in zip(&create.args.args, user_def) {
                                fields.insert(
                                    field_name.to_owned(),
                                    interpret_operand_expression(arg, self, vr)?,
                                );
                            }

                            Data::DataObject(DataObject::StructObject(StructObject(fields)))
                        }
                    }
                } else {
                    return Err(RuntimeError(
                        "Did not find structure definition".to_string(),
                        FileLocation::None,
                    ));
                }
            }
        };

        self.data.insert(key, DataCase { ref_count: 0, data });

        return Ok(key);
    }
}

enum ExitMethod {
    ImplicitNullReturn,
    ExplicitReturn(u32),
    LoopContinue,
    LoopBreak,
}

fn interpret_operand_expression(
    operand_expression: &OperandExpression,
    gc: &mut GarbageCollector,
    vr: &VariableRegistry,
) -> Result<u32, RuntimeError> {
    match operand_expression {
        OperandExpression::Unary { operand, val } => todo!(),
        OperandExpression::Binary {
            operand,
            left,
            right,
        } => todo!(),
        OperandExpression::Literal(literal) => gc.add_literal(literal),
        OperandExpression::Object(object) => gc.resolve_object(object, vr),
        OperandExpression::Create(create) => gc.create_object(create, vr),
    }
}

fn run_term_block(
    block: &TermBlock,
    gc: &mut GarbageCollector,
    vr: &VariableRegistry,
) -> Result<ExitMethod, RuntimeError> {
    let mut vr = vr.create_child();

    for term in &block.terms {
        match term {
            Term::Print { ln, operand_block } => {
                let value = interpret_operand_expression(operand_block, gc, &vr)?;
                let output = gc.id_to_string(value);
                if *ln {
                    print!("{output}\n");
                } else {
                    print!("{output}")
                }
            }
            Term::DeclareVar { name, value, .. } => {
                let key = interpret_operand_expression(value, gc, &vr)?;
                vr.vars.insert(name.clone(), key);
                gc.data.get_mut(&key).unwrap().ref_count += 1;
            }
            Term::Return { value } => {
                return Ok(ExitMethod::ExplicitReturn(interpret_operand_expression(
                    value, gc, &vr,
                )?));
            }
            Term::UpdateVar {
                var,
                set_operator,
                value,
            } => {
                let key = gc.resolve_object(var, &vr);
            }
            Term::If {
                conditional,
                block,
                else_block,
            } => {
                let operand_result = interpret_operand_expression(conditional, gc, &vr)?;
                let result = gc.id_to_bool(operand_result);
                if result {
                    let output = run_term_block(block, gc, &vr)?;
                    if let ExitMethod::ExplicitReturn(..) = output {
                        return Ok(output);
                    }
                } else {
                    let output = run_term_block(else_block, gc, &vr)?;
                    if let ExitMethod::ExplicitReturn(..) = output {
                        return Ok(output);
                    }
                }
            }
            Term::Loop {
                counter,
                conditional,
                block,
            } => {
                let mut vr = vr.create_child();
                let data = Data::DataObject(DataObject::RootObject(RootObject::Int(0)));
                let data_case = DataCase { ref_count: 1, data };
                let key = random();
                gc.data.insert(key, data_case);
                vr.add_var(counter, key);

                let mut operand_result = interpret_operand_expression(conditional, gc, &vr)?;
                while gc.id_to_bool(operand_result) {
                    match run_term_block(block, gc, &vr)? {
                        ExitMethod::ExplicitReturn(result) => {
                            return Ok(ExitMethod::ExplicitReturn(result))
                        }
                        ExitMethod::LoopBreak => {
                            break;
                        }
                        _ => {
                            if let Data::DataObject(DataObject::RootObject(RootObject::Int(
                                ref mut counter,
                            ))) = gc.data.get_mut(&key).unwrap().data
                            {
                                *counter += 1;
                            }
                        }
                    }

                    operand_result = interpret_operand_expression(conditional, gc, &vr)?;
                }

                vr.release(gc);
            }
            Term::Break => return Ok(ExitMethod::LoopBreak),
            Term::Continue => return Ok(ExitMethod::LoopContinue),
            Term::Call { value } => {
                interpret_operand_expression(value, gc, &vr)?;
            }
        }
    }

    vr.release(gc);
    return Ok(ExitMethod::ImplicitNullReturn);
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let mut gc = GarbageCollector::new();

    for _struct in program.structs {
        gc.add_struct(_struct)?;
    }

    for func in program.functions {
        gc.add_function(func)?;
    }

    let result = match gc.root_var_registry.vars.get(PROGRAM_ENTRY) {
        Some(main_id) => match &gc.data[main_id].data {
            Data::FuncDef(func) => {
                let func = func.clone();
                func.call(vec![gc.command_line_args], &mut gc)
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
    };

    return match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };
}
