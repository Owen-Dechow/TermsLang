mod garbage_collector;
mod internal_funcs;
mod syntax;
mod var_registry;

use internal_funcs::{self as infn};
use var_registry::VariableRegistry;

use std::{
    collections::{HashMap, HashSet},
    iter::zip,
    rc::Rc,
};
use syntax::{self as syn};

use garbage_collector::GarbageCollector;

use crate::{
    errors::{FileLocation, RuntimeError},
    lexer::tokens::{Operator, TokenType},
    parser::{parse_operand_block::OperandExpression, Program, Term, TermBlock},
};

struct Counter(u32);
impl Counter {
    pub fn new_key(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}

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
    fn get_root_type_def(&self, gc: &GarbageCollector) -> Rc<StructDef> {
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
    self_key: u32,
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
        self_ptr: u32,
        gc: &mut GarbageCollector,
        method: &String,
        args: Vec<u32>,
        root_vr: &VariableRegistry,
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        let result = match gc.objects[&self_ptr].borrow_inner() {
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

                return func_def.struct_call(args, gc, struct_object.self_key, root_vr, counter);
            }
            DataObject::RootObject(root) => match method.as_str() {
                syn::ADD_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    method,
                    &infn::add_roots,
                    counter,
                ),

                syn::SUBTRACT_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::subtract_roots,
                    counter,
                ),
                syn::MULTIPLY_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::multiply_roots,
                    counter,
                ),
                syn::DIVIDE_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::divide_roots,
                    counter,
                ),
                syn::MODULO_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::modulo_roots,
                    counter,
                ),
                syn::EXPONENT_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::exponent_roots,
                    counter,
                ),
                syn::GREATER_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::greater_roots,
                    counter,
                ),
                syn::GREATER_OR_EQUAL_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::greater_or_equal_roots,
                    counter,
                ),
                syn::LESS_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::less_roots,
                    counter,
                ),
                syn::LESS_OR_EQUAL_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::less_or_equal_roots,
                    counter,
                ),
                syn::OR_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::or_roots,
                    counter,
                ),
                syn::AND_FUNC => infn::std_binary_operation(
                    self_ptr,
                    gc,
                    &args,
                    &method,
                    &infn::and_roots,
                    counter,
                ),

                syn::EQUAL_FUNC => infn::equal(self_ptr, gc, &args, counter),
                syn::TO_STRING_FUNC => infn::to_string(self_ptr, gc, counter),
                syn::TO_INT_FUNC => infn::to_int(self_ptr, gc, counter),
                syn::TO_FLOAT_FUNC => todo!(),
                _ => {
                    return Err(RuntimeError(
                        format!(
                            "No function {} found for root type {}",
                            method,
                            match *root.get_root_type_def(gc) {
                                StructDef::User { ref name, .. } => name,
                                StructDef::Root { ref name, .. } => name,
                            }
                        ),
                        FileLocation::None,
                    ))
                }
            },
            DataObject::ArrayObject(..) => todo!(),
        };

        return result;
    }

    fn call_array_func(
        id: u32,
        gc: &mut GarbageCollector,
        method: &String,
        args: Vec<u32>,
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        let result = match method.as_str() {
            syn::REMOVE_FUNC => infn::remove(id, gc, &args, counter),
            syn::APPEND_FUNC => infn::append(id, gc, &args, counter),
            syn::LEN_FUNC => infn::len(id, gc, counter),
            syn::INDEX_FUNC => infn::index(id, gc, &args),
            _ => {
                return Err(RuntimeError(
                    format!("No function {} found for array", method),
                    FileLocation::None,
                ))
            }
        };

        return result;
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
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        match self {
            RootFunc::ReadLn => infn::readln(gc, counter),
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
    fn global_call(
        &self,
        args: Vec<u32>,
        gc: &mut GarbageCollector,
        root_vr: &VariableRegistry,
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        let mut vr = root_vr.create_child();
        self.call(args, gc, &mut vr, root_vr, counter)
    }

    fn struct_call(
        &self,
        args: Vec<u32>,
        gc: &mut GarbageCollector,
        _struct: u32,
        root_vr: &VariableRegistry,
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        let mut vr = root_vr.create_child();
        vr.vars.insert(syn::STRUCT_SELF.to_owned(), _struct);
        let mut vr = vr.create_child();

        self.call(args, gc, &mut vr, root_vr, counter)
    }

    fn call(
        &self,
        args: Vec<u32>,
        gc: &mut GarbageCollector,
        vr: &mut VariableRegistry,
        root_vr: &VariableRegistry,
        counter: &mut Counter,
    ) -> Result<ExitMethod, RuntimeError> {
        if args.len() != self.args.len() {
            return Err(RuntimeError(
                format!("Function call to {} has invalid arguments.", self.name),
                FileLocation::None,
            ));
        }

        for (ref_id, (arg_name, _arg_type)) in zip(&args, &self.args) {
            let data_case = gc.objects.get_mut(&ref_id).unwrap();
            if let DataObject::RootObject(root_object) = &data_case.data {
                if data_case.ref_count > 0 {
                    let root_object = root_object.clone();
                    let key = gc.insert_data(DataObject::RootObject(root_object), counter, 0);
                    vr.vars.insert(arg_name.clone(), key);
                } else {
                    data_case.ref_count += 1;
                    vr.vars.insert(arg_name.clone(), *ref_id);
                }
            } else {
                data_case.ref_count += 1;
                vr.vars.insert(arg_name.clone(), *ref_id);
            }
        }

        let result = match &self.block {
            FuncBlock::User(term_block) => run_term_block(term_block, gc, vr, root_vr, counter)?,
            FuncBlock::Root(root) => root.call(gc, &vr, counter)?,
        };

        if let ExitMethod::ExplicitReturn(key) = result {
            vr.release_exclude(gc, &key);
        } else {
            vr.release(gc);
        }

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
    Array(Box<TypeResolve>, FileLocation),
    Standard(u32, FileLocation),
}

#[derive(Debug)]
struct DataCase {
    ref_count: u32,
    data: DataObject,
}
impl DataCase {
    pub fn borrow_inner(&self) -> &DataObject {
        &self.data
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
    vr: &mut VariableRegistry,
    root_vr: &VariableRegistry,
    counter: &mut Counter,
) -> Result<u32, RuntimeError> {
    let result = match operand_expression {
        OperandExpression::Unary { .. } => todo!(),
        OperandExpression::Binary {
            operand,
            left,
            right,
        } => {
            if let TokenType::Operator(operator) = &operand.0 {
                let left = interpret_operand_expression(left, gc, vr, root_vr, counter)?;
                let right = interpret_operand_expression(right, gc, vr, root_vr, counter)?;

                let result = match DataObject::call_method(
                    left,
                    gc,
                    &syn::convert_operator(operator),
                    vec![right],
                    root_vr,
                    counter,
                )? {
                    ExitMethod::ImplicitNullReturn => todo!(),
                    ExitMethod::ExplicitReturn(id) => id,
                    ExitMethod::LoopContinue => todo!(),
                    ExitMethod::LoopBreak => todo!(),
                };

                gc.release(left);
                gc.release(right);
                return Ok(result);
            } else {
                panic!("Should be operand");
            }
        }
        OperandExpression::Literal(literal) => gc.add_literal(literal, counter),
        OperandExpression::Object(object) => {
            let obj = gc.resolve_object(object, vr, root_vr, counter)?;

            if obj == gc.root_type_map[&RootType::Null] {
                return Ok(gc.null_object(counter));
            } else {
                return Ok(obj);
            }
        }
        OperandExpression::Create(create) => gc.create_object(create, vr, root_vr, counter),
        OperandExpression::Dot { left, right } => {
            let parent = interpret_operand_expression(&left, gc, vr, root_vr, counter)?;
            let obj = gc.resolve_sub_object(parent, None, right, vr, root_vr, counter)?;
            gc.release(parent);
            if obj == gc.root_type_map[&RootType::Null] {
                return Ok(gc.null_object(counter));
            } else {
                return Ok(obj);
            }
        }
    };

    return result;
}

fn run_term_block(
    block: &TermBlock,
    gc: &mut GarbageCollector,
    vr: &mut VariableRegistry,
    root_vr: &VariableRegistry,
    counter: &mut Counter,
) -> Result<ExitMethod, RuntimeError> {
    let mut vr = vr.create_child();
    for term in &block.terms {
        match term {
            Term::Print { ln, operand_block } => {
                let value =
                    interpret_operand_expression(operand_block, gc, &mut vr, root_vr, counter)?;
                let output = match gc.objects[&value].borrow_inner() {
                    DataObject::RootObject(RootObject::String(string)) => string,
                    _ => {
                        return Err(RuntimeError(
                            "Cannot print non string objects".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                if *ln {
                    println!("{output}");
                } else {
                    print!("{output}")
                }
                gc.release(value);
            }
            Term::DeclareVar { name, value, .. } => {
                let key = interpret_operand_expression(value, gc, &mut vr, root_vr, counter)?;
                vr.vars.insert(name.clone(), key);
                match gc.objects.get_mut(&key) {
                    Some(datacase) => datacase.ref_count += 1,
                    None => {
                        if gc.global_methods.contains_key(&key) {
                            return Err(RuntimeError(
                                "Cannot assign function definition to variable".to_string(),
                                FileLocation::None,
                            ));
                        } else if gc.global_structs.contains_key(&key) {
                            return Err(RuntimeError(
                                "Cannot assign struct definition to variable".to_string(),
                                FileLocation::None,
                            ));
                        } else {
                            todo!()
                        }
                    }
                }
            }
            Term::Return { value } => {
                let result = interpret_operand_expression(value, gc, &mut vr, root_vr, counter)?;
                vr.release(gc);

                return Ok(ExitMethod::ExplicitReturn(result));
            }
            Term::UpdateVar {
                var,
                set_operator,
                value,
            } => {
                let key = gc.resolve_object(var, &mut vr, root_vr, counter)?;
                match set_operator {
                    Operator::Set => {
                        let value =
                            interpret_operand_expression(value, gc, &mut vr, root_vr, counter)?;
                        gc.objects.get_mut(&key).unwrap().data = gc.objects[&value].data.clone();
                        gc.release(value);
                    }
                    _ => {
                        let value =
                            interpret_operand_expression(value, gc, &mut vr, root_vr, counter)?;
                        let function = match set_operator {
                            Operator::SetAdd => syn::ADD_FUNC,
                            Operator::SetSubtract => syn::SUBTRACT_FUNC,
                            Operator::SetMultiply => syn::MULTIPLY_FUNC,
                            Operator::SetDivide => syn::DIVIDE_FUNC,
                            Operator::SetModulo => syn::MODULO_FUNC,
                            Operator::SetExponent => syn::EXPONENT_FUNC,
                            _ => todo!(),
                        };

                        let value = DataObject::call_method(
                            key,
                            gc,
                            &function.to_owned(),
                            vec![value],
                            root_vr,
                            counter,
                        )?;
                        match value {
                            ExitMethod::ExplicitReturn(val) => {
                                gc.objects.get_mut(&key).unwrap().data =
                                    gc.objects[&val].data.clone();
                                gc.release(val);
                            }
                            _ => todo!(),
                        }
                    }
                }
            }
            Term::If {
                conditional,
                block,
                else_block,
            } => {
                let operand_result =
                    interpret_operand_expression(conditional, gc, &mut vr, root_vr, counter)?;
                let result = *match gc.objects[&operand_result].borrow_inner() {
                    DataObject::RootObject(RootObject::Bool(_bool)) => _bool,
                    _ => {
                        return Err(RuntimeError(
                            "Conditional was not a boolean".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                gc.release(operand_result);
                if result {
                    let output = run_term_block(block, gc, &mut vr, root_vr, counter)?;

                    if output != ExitMethod::ImplicitNullReturn {
                        return Ok(output);
                    }
                } else {
                    let output = run_term_block(else_block, gc, &mut vr, root_vr, counter)?;

                    if output != ExitMethod::ImplicitNullReturn {
                        return Ok(output);
                    }
                }
            }
            Term::Loop {
                counter: counter_var,
                conditional,
                block,
            } => {
                let mut vr = vr.create_child();
                let data = DataObject::RootObject(RootObject::Int(0));
                let key = gc.insert_data(data, counter, 1);
                vr.add_var(counter_var, key);

                let operand_result =
                    interpret_operand_expression(conditional, gc, &mut vr, root_vr, counter)?;
                let mut conditional_result = match gc.objects[&operand_result].borrow_inner() {
                    DataObject::RootObject(RootObject::Bool(_bool)) => *_bool,
                    _ => {
                        return Err(RuntimeError(
                            "Loop conditional was not a boolean".to_string(),
                            FileLocation::None,
                        ))
                    }
                };
                gc.release(operand_result);
                while conditional_result {
                    let result = run_term_block(block, gc, &mut vr, root_vr, counter)?;
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

                            let operand_result = interpret_operand_expression(
                                conditional,
                                gc,
                                &mut vr,
                                root_vr,
                                counter,
                            )?;
                            conditional_result = match gc.objects[&operand_result].borrow_inner() {
                                DataObject::RootObject(RootObject::Bool(_bool)) => *_bool,
                                _ => {
                                    return Err(RuntimeError(
                                        "Loop conditional was not a boolean".to_string(),
                                        FileLocation::None,
                                    ))
                                }
                            };
                            gc.release(operand_result);
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
                let result = interpret_operand_expression(value, gc, &mut vr, root_vr, counter)?;
                gc.release(result);
            }
        };
    }

    vr.release(gc);
    return Ok(ExitMethod::ImplicitNullReturn);
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let mut root_vr = VariableRegistry {
        vars: HashMap::new(),
        parent: None,
    };

    let mut counter = Counter(0);

    let mut gc = GarbageCollector::new(&mut root_vr, &mut counter);

    for _struct in program.structs {
        gc.add_struct(_struct, &mut root_vr, &mut counter)?;
    }

    for func in program.functions {
        gc.add_function(func, &mut root_vr, &mut counter)?;
    }

    let result = match root_vr.vars.get(syn::PROGRAM_ENTRY) {
        Some(main_id) => match gc.global_methods.get(main_id) {
            Some(func) => {
                let func = func.clone();
                func.global_call(vec![gc.command_line_args], &mut gc, &root_vr, &mut counter)
            }
            None => Err(RuntimeError(
                format!("{} must be a function.", syn::PROGRAM_ENTRY),
                FileLocation::None,
            )),
        },
        None => Err(RuntimeError(
            format!("{} function not found.", syn::PROGRAM_ENTRY),
            FileLocation::None,
        )),
    };
    return match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };
}
