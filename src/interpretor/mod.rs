mod garbage_collector;
mod internal_funcs;
mod syntax;

use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

use garbage_collector::GarbageCollector;
use rand::random;

use crate::{
    errors::{FileLocation, RuntimeError},
    lexer::tokens::{Operator, TokenType},
    parser::{parse_operand_block::OperandExpression, Program, Term, TermBlock},
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
impl RootObject {
    fn get_root_type_def(&self, gc: &GarbageCollector) -> StructDef {
        gc.global_structs[&gc.root_type_map[&match self {
            RootObject::String(_) => RootType::String,
            RootObject::Int(_) => RootType::Int,
            RootObject::Float(_) => RootType::Float,
            RootObject::Bool(_) => RootType::Bool,
            RootObject::Null => RootType::Null,
        }]]
            .clone()
    }
}

#[derive(Debug, Clone)]
struct StructObject {
    _type: u32,
    fields: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
struct ArrayObject(Vec<u32>);

#[derive(Debug, Clone)]
enum DataObject {
    StructObject(StructObject),
    RootObject(RootObject),
    ArrayObject(ArrayObject),
}
impl DataObject {
    fn call_method(
        &self,
        gc: &mut GarbageCollector,
        method: &String,
        args: Vec<u32>,
    ) -> Result<ExitMethod, RuntimeError> {
        match self {
            DataObject::StructObject(struct_object) => {
                let struct_def = gc.get_struct_def_from_id(struct_object._type)?;
                let func_id = match struct_def {
                    StructDef::User { methods, .. } => methods.get(method),
                    StructDef::Root { _type, .. } => todo!(),
                };

                let func_def = match func_id {
                    Some(id) => gc.global_methods[id].clone(),
                    None => {
                        return Err(RuntimeError(
                            format!(
                                "No function {} found for struct {}",
                                method,
                                match struct_def {
                                    StructDef::User { name, .. } => name,
                                    StructDef::Root { name, .. } => name,
                                }
                            ),
                            FileLocation::None,
                        ))
                    }
                };

                return func_def.call(args, gc);
            }
            DataObject::RootObject(root) => match method.as_str() {
                syntax::ADD_FUNC => internal_funcs::add(root, gc, args),
                syntax::LESS_FUNC => internal_funcs::less(root, gc, args),
                syntax::TO_STRING_FUNC => internal_funcs::to_string(root, gc),
                syntax::MODULO_FUNC => internal_funcs::modulo(root, gc, args),
                syntax::EQUAL_FUNC => internal_funcs::equal(root, gc, args),
                _ => {
                    return Err(RuntimeError(
                        format!(
                            "No function {} found for struct {}",
                            method,
                            match root.get_root_type_def(gc) {
                                StructDef::User { name, .. } => name,
                                StructDef::Root { name, .. } => name,
                            }
                        ),
                        FileLocation::None,
                    ))
                }
            },
            DataObject::ArrayObject(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
enum StructDef {
    User {
        name: String,
        properties: HashMap<String, TypeResolve>,
        methods: HashMap<String, u32>,
    },
    Root {
        name: String,
        methods: HashSet<String>,
        _type: RootType,
    },
}

#[derive(Debug, Clone)]
enum RootFunc {
    ReadLn,
}
impl RootFunc {
    fn call(
        &self,
        gc: &mut GarbageCollector,
        _vr: &VariableRegistry,
    ) -> Result<ExitMethod, RuntimeError> {
        match self {
            RootFunc::ReadLn => internal_funcs::readln(gc),
        }
    }
}

#[derive(Debug, Clone)]
enum FuncBlock {
    User(TermBlock),
    Root(RootFunc),
}

#[derive(Debug, Clone)]
struct FuncDef {
    name: String,
    args: Vec<(String, TypeResolve)>,
    block: FuncBlock,
}
impl FuncDef {
    fn call(&self, args: Vec<u32>, gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
        let mut vr = gc.new_function_scope();
        if args.len() != self.args.len() {
            return Err(RuntimeError(
                format!("Function call to {} has invalid arguments.", self.name),
                FileLocation::None,
            ));
        }

        for (ref_id, (arg_name, _arg_type)) in zip(&args, &self.args) {
            let data_case = gc.objects.get_mut(&ref_id).unwrap();
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

#[derive(Debug)]
struct DataCase {
    ref_count: u32,
    data: DataObject,
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
            let data_case = gc.objects.get_mut(&ref_id).unwrap();
            data_case.ref_count -= 1;
            if data_case.ref_count == 0 {
                gc.objects.remove(&ref_id);
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

#[derive(Debug, PartialEq)]
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
        OperandExpression::Unary { .. } => todo!(),
        OperandExpression::Binary {
            operand,
            left,
            right,
        } => {
            if let TokenType::Operator(operator) = &operand.0 {
                let left = interpret_operand_expression(left, gc, vr)?;
                let left = gc.objects.get(&left).unwrap().data.clone();

                let right = interpret_operand_expression(right, gc, vr)?;

                let result =
                    match left.call_method(gc, &syntax::convert_operator(operator), vec![right])? {
                        ExitMethod::ImplicitNullReturn => todo!(),
                        ExitMethod::ExplicitReturn(id) => id,
                        ExitMethod::LoopContinue => todo!(),
                        ExitMethod::LoopBreak => todo!(),
                    };

                return Ok(result);
            } else {
                Err(RuntimeError(
                    format!("Expected Operand"),
                    FileLocation::None,
                ))
            }
        }
        OperandExpression::Literal(literal) => gc.add_literal(literal),
        OperandExpression::Object(object) => {
            let obj = gc.resolve_object(object, vr)?;
            if obj == gc.root_type_map[&RootType::Null] {
                return Ok(gc.create_null_object());
            } else {
                return Ok(obj);
            }
        }
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
                let output = match gc.objects.get(&value) {
                    Some(DataCase {
                        data: DataObject::RootObject(RootObject::String(string)),
                        ..
                    }) => string,
                    _ => {
                        return Err(RuntimeError(
                            "Cannot print non string objects".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                if *ln {
                    print!("{output}\n");
                } else {
                    print!("{output}")
                }
            }
            Term::DeclareVar { name, value, .. } => {
                let key = interpret_operand_expression(value, gc, &vr)?;
                vr.vars.insert(name.clone(), key);
                match gc.objects.get_mut(&key) {
                    Some(datacase) => datacase.ref_count += 1,
                    None => {
                        return Err(RuntimeError(
                            "Cannot assign struct or function definition to variable".to_string(),
                            FileLocation::None,
                        ))
                    }
                }
            }
            Term::Return { value } => {
                let result = interpret_operand_expression(value, gc, &vr)?;
                vr.release(gc);
                return Ok(ExitMethod::ExplicitReturn(result));
            }
            Term::UpdateVar {
                var,
                set_operator,
                value,
            } => {
                let key = gc.resolve_object(var, &vr)?;
                match set_operator {
                    Operator::Set => {
                        let value = interpret_operand_expression(value, gc, &vr)?;
                        gc.objects.get_mut(&key).unwrap().data = gc.objects[&value].data.clone();
                    }
                    _ => todo!(),
                }
            }
            Term::If {
                conditional,
                block,
                else_block,
            } => {
                let operand_result = interpret_operand_expression(conditional, gc, &vr)?;
                let result = match gc.objects.get(&operand_result) {
                    Some(DataCase {
                        data: DataObject::RootObject(RootObject::Bool(_bool)),
                        ..
                    }) => _bool,
                    _ => {
                        return Err(RuntimeError(
                            "Conditional was not a boolean".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                if *result {
                    let output = run_term_block(block, gc, &vr)?;

                    if output != ExitMethod::ImplicitNullReturn {
                        return Ok(output);
                    }
                } else {
                    let output = run_term_block(else_block, gc, &vr)?;

                    if output != ExitMethod::ImplicitNullReturn {
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
                let data = DataObject::RootObject(RootObject::Int(0));
                let data_case = DataCase { ref_count: 1, data };
                let key = random();
                gc.objects.insert(key, data_case);
                vr.add_var(counter, key);

                let operand_result = interpret_operand_expression(conditional, gc, &vr)?;
                let mut conditional_result = match gc.objects.get(&operand_result) {
                    Some(DataCase {
                        data: DataObject::RootObject(RootObject::Bool(_bool)),
                        ..
                    }) => *_bool,
                    _ => {
                        return Err(RuntimeError(
                            "Loop conditional was not a boolean".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                while conditional_result {
                    let result = run_term_block(block, gc, &vr)?;
                    match result {
                        ExitMethod::ExplicitReturn(result) => {
                            return Ok(ExitMethod::ExplicitReturn(result))
                        }
                        ExitMethod::LoopBreak => {
                            conditional_result = false;
                        }
                        _ => {
                            if let DataObject::RootObject(RootObject::Int(ref mut counter)) =
                                gc.objects.get_mut(&key).unwrap().data
                            {
                                *counter += 1;
                            }

                            let operand_result =
                                interpret_operand_expression(conditional, gc, &vr)?;
                            conditional_result = match gc.objects.get(&operand_result) {
                                Some(DataCase {
                                    data: DataObject::RootObject(RootObject::Bool(_bool)),
                                    ..
                                }) => *_bool,
                                _ => {
                                    return Err(RuntimeError(
                                        "Loop conditional was not a boolean".to_string(),
                                        FileLocation::None,
                                    ))
                                }
                            };
                        }
                    }
                }

                vr.release(gc);
            }
            Term::Break => {
                vr.release(gc);
                return Ok(ExitMethod::LoopBreak);
            }
            Term::Continue => {
                vr.release(gc);
                return Ok(ExitMethod::LoopContinue);
            }
            Term::Call { value } => {
                interpret_operand_expression(value, gc, &vr)?;
            }
        };
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

    let result = match gc.root_var_registry.vars.get(syntax::PROGRAM_ENTRY) {
        Some(main_id) => match gc.global_methods.get(main_id) {
            Some(func) => {
                let func = func.clone();
                func.call(vec![gc.command_line_args], &mut gc)
            }
            None => Err(RuntimeError(
                format!("{} must be a function.", syntax::PROGRAM_ENTRY),
                FileLocation::None,
            )),
        },
        None => Err(RuntimeError(
            format!("{} function not found.", syntax::PROGRAM_ENTRY),
            FileLocation::None,
        )),
    };

    return match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };
}
