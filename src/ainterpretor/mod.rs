mod internal_funcs;

use crate::rc_ref;
use crate::{
    active_parser::{
        names as nm, AFunc, AFuncBlock, ALiteral, AObject, AObjectType, AOperandExpression,
        AOperandExpressionValue, AProgram, ARootTypeCollection, AStruct, ATerm, ATermBlock, AType,
    },
    cli::Args,
    errors::RuntimeError,
};
use std::{cell::RefCell, collections::HashMap, iter::zip, rc::Rc};

struct GlobalData {
    structs: HashMap<String, Rc<AStruct>>,
    functions: HashMap<String, Rc<AFunc>>,
    root_types: ARootTypeCollection,
}
impl GlobalData {
    fn new(root_types: ARootTypeCollection) -> Self {
        Self {
            structs: HashMap::new(),
            functions: HashMap::new(),
            root_types,
        }
    }

    fn add_struct(&mut self, _struct: Rc<AStruct>) {
        self.structs.insert(_struct.name.clone(), _struct);
    }

    fn add_function(&mut self, func: Rc<AFunc>) {
        self.functions.insert(func.name.clone(), func);
    }

    fn resolve_aobject(
        &self,
        aobject: &AObject,
        ds: &DataScope,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        match &aobject.kind {
            AObjectType::Identity(id) => match self.structs.get(id) {
                Some(_struct) => match aobject.sub {
                    Some(_) => panic!(),
                    None => Ok(rc_ref!(Data::StructDef(_struct.clone()))),
                },
                None => match *aobject._type.borrow() {
                    AType::FuncDefRef(ref func) => match &aobject.sub {
                        Some(sub) => Func(func.clone()).resolve_aobject(&sub, None, ds, self),
                        None => Ok(rc_ref!(Data::FuncDef(Func(func.clone())))),
                    },
                    _ => panic!(),
                },
            },
            AObjectType::Call(..) => panic!(),
        }
    }

    fn null(&self) -> Rc<RefCell<Data>> {
        rc_ref!(Data::StructObject(StructObject::Root(Root {
            _type: AType::from_astruct(self.root_types.null_type.clone())
                .borrow()
                .to_type_instance(),
            value: RootValue::Null,
        })))
    }
}

#[derive(Debug)]
struct Func(Rc<AFunc>);
impl Func {
    fn resolve_aobject(
        &self,
        aobject: &AObject,
        connected_instance: Option<Rc<RefCell<Data>>>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        match &aobject.kind {
            AObjectType::Identity(..) => panic!(),
            AObjectType::Call(acall) => {
                let mut args = Vec::new();
                for arg in &acall.args {
                    args.push(interpret_operand_expression(&arg, ds, gd)?);
                }

                let mut func_ds = DataScope::new();
                if let Some(inst) = connected_instance {
                    func_ds.insert_data(nm::THIS, inst);
                }

                let result = match interpret_function(&self.0, Some(&func_ds), gd, &args)? {
                    BlockExit::ImplicitNull => gd.null(),
                    BlockExit::Explicit(rc) => rc,
                    _ => panic!(),
                };

                return match &aobject.sub {
                    Some(sub) => {
                        result
                            .borrow()
                            .resolve_aobject(&sub, Some(result.clone()), &ds, gd)
                    }
                    None => Ok(result),
                };
            }
        }
    }
}

#[derive(Debug)]
enum Data {
    StructObject(StructObject),
    ArrayObject(Array),
    FuncDef(Func),
    StructDef(Rc<AStruct>),
}
impl Data {
    fn from_aliteral(aliteral: &ALiteral, gd: &GlobalData) -> Rc<RefCell<Self>> {
        let (value, _type) = match aliteral {
            ALiteral::Int(int) => (RootValue::Int(*int), gd.root_types.int_type.clone()),
            ALiteral::Float(float) => (RootValue::Float(*float), gd.root_types.float_type.clone()),
            ALiteral::String(string) => (
                RootValue::String(string.to_string()),
                gd.root_types.string_type.clone(),
            ),
            ALiteral::Bool(b) => (RootValue::Bool(*b), gd.root_types.bool_type.clone()),
        };

        let data = Data::StructObject(StructObject::Root(Root {
            _type: AType::from_astruct(_type).borrow().to_type_instance(),
            value,
        }));

        return rc_ref!(data);
    }

    fn resolve_aobject(
        &self,
        aobject: &AObject,
        connected_instance: Option<Rc<RefCell<Data>>>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Self>>, RuntimeError> {
        match self {
            Data::StructObject(struct_object) => {
                struct_object.resolve_aobject(aobject, connected_instance, ds, gd)
            }
            Data::ArrayObject(arr) => arr.resolve_aobject(aobject, connected_instance, ds, gd),
            Data::FuncDef(func) => func.resolve_aobject(aobject, connected_instance, ds, gd),
            Data::StructDef(..) => panic!(),
        }
    }

    fn create_new(
        _type: Rc<RefCell<AType>>,
        args: &Vec<AOperandExpression>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Self>>, RuntimeError> {
        match *_type.borrow() {
            AType::ArrayObject(..) => Ok(rc_ref!(Data::ArrayObject(Array(rc_ref!(Vec::new()))))),
            AType::StructDefRef(ref astruct) => match astruct.root {
                true => {
                    let data = rc_ref!(Data::StructObject(StructObject::Root(Root {
                        _type: _type.borrow().to_type_instance(),
                        value: RootValue::Null,
                    })));

                    let mut func_args = Vec::new();
                    for arg in args {
                        func_args.push(interpret_operand_expression(arg, ds, gd)?);
                    }

                    let mut func_ds = DataScope::new();
                    func_ds.insert_data(nm::THIS, data.clone());

                    interpret_function(
                        &astruct.methods[nm::F_NEW],
                        Some(&func_ds),
                        gd,
                        &func_args,
                    )?;

                    return Ok(data);
                }
                false => {
                    let mut data = HashMap::new();
                    for field in &astruct.fields {
                        data.insert(field.0.clone(), gd.null());
                    }

                    let struct_object = StructObject::User {
                        _type: astruct.clone(),
                        data,
                    };

                    let data = rc_ref!(Data::StructObject(struct_object));

                    if let Some(func) = astruct.methods.get(nm::F_NEW) {
                        let mut func_ds = DataScope::new();
                        func_ds.insert_data(nm::THIS, data.clone());

                        let mut func_args = Vec::new();
                        for arg in args {
                            func_args.push(interpret_operand_expression(arg, ds, gd)?);
                        }

                        interpret_function(func, Some(&func_ds), gd, &func_args)?;
                    }

                    return Ok(data);
                }
            },
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
enum StructObject {
    User {
        _type: Rc<AStruct>,
        data: HashMap<String, Rc<RefCell<Data>>>,
    },
    Root(Root),
}
impl StructObject {
    fn resolve_aobject(
        &self,
        aobject: &AObject,
        connected_instance: Option<Rc<RefCell<Data>>>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        match self {
            StructObject::User { _type, data } => match &aobject.kind {
                AObjectType::Identity(id) => match data.get(id) {
                    Some(data) => match &aobject.sub {
                        Some(sub) => data
                            .borrow()
                            .resolve_aobject(sub, Some(data.clone()), ds, gd),
                        None => Ok(data.clone()),
                    },
                    None => match &aobject.sub {
                        Some(sub) => Func(_type.methods[id].clone()).resolve_aobject(
                            &sub,
                            connected_instance,
                            ds,
                            gd,
                        ),
                        None => Ok(rc_ref!(Data::FuncDef(Func(_type.methods[id].clone())))),
                    },
                },
                AObjectType::Call(..) => panic!(),
            },
            StructObject::Root(root) => root.resolve_aobject(aobject, connected_instance, ds, gd),
        }
    }
}

#[derive(Debug, Clone)]
struct Root {
    _type: Rc<RefCell<AType>>,
    value: RootValue,
}
impl Root {
    fn resolve_aobject(
        &self,
        aobject: &AObject,
        connected_instance: Option<Rc<RefCell<Data>>>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        match &aobject.kind {
            AObjectType::Identity(id) => {
                let func = match *self._type.borrow() {
                    AType::StructObject(ref astruct) => astruct.methods[id].clone(),
                    _ => panic!("{:?}", self),
                };

                let func = Func(func);

                match &aobject.sub {
                    Some(sub) => func.resolve_aobject(&sub, connected_instance, ds, gd),
                    None => Ok(rc_ref!(Data::FuncDef(func))),
                }
            }
            AObjectType::Call(..) => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
enum RootValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Null,
}

#[derive(Debug)]
struct Array(Rc<RefCell<Vec<Rc<RefCell<Data>>>>>);
impl Array {
    fn resolve_aobject(
        &self,
        aobject: &AObject,
        connected_instance: Option<Rc<RefCell<Data>>>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        let func = match *aobject._type.borrow() {
            AType::FuncDefRef(ref func) => func.clone(),
            _ => panic!(),
        };

        let data = Data::FuncDef(Func(func));

        match &aobject.sub {
            Some(sub) => data.resolve_aobject(&sub, connected_instance, ds, gd),
            None => Ok(rc_ref!(data)),
        }
    }
}

pub struct DataScope<'a> {
    parent: Option<&'a DataScope<'a>>,
    data: HashMap<String, Rc<RefCell<Data>>>,
}
impl<'a> DataScope<'a> {
    fn new() -> Self {
        Self {
            parent: None,
            data: HashMap::new(),
        }
    }

    fn create_child(&'a self) -> Self {
        Self {
            parent: Some(self),
            data: HashMap::new(),
        }
    }

    fn insert_data(&mut self, name: &str, data: Rc<RefCell<Data>>) {
        self.data.insert(name.to_string(), data);
    }

    fn add_arglist(&mut self, args: &[Rc<RefCell<Data>>], func: &AFunc) {
        for arg in zip(&func.args, args) {
            self.insert_data(&arg.0.name, arg.1.clone());
        }
    }

    fn resolve_aobject(
        &self,
        aobject: &AObject,
        gd: &GlobalData,
        ds: &DataScope,
    ) -> Result<Rc<RefCell<Data>>, RuntimeError> {
        match &aobject.kind {
            AObjectType::Identity(id) => match self.data.get(id) {
                Some(obj) => match &aobject.sub {
                    Some(sub) => obj.borrow().resolve_aobject(sub, Some(obj.clone()), ds, gd),
                    None => return Ok(obj.clone()),
                },
                None => match self.parent {
                    Some(parent) => parent.resolve_aobject(aobject, gd, ds),
                    None => gd.resolve_aobject(aobject, ds),
                },
            },
            _ => panic!(),
        }
    }
}

enum BlockExit {
    Continue,
    Break,
    ImplicitNull,
    Explicit(Rc<RefCell<Data>>),
}

#[macro_export]
macro_rules! rc_ref {
    ($inside:expr) => {
        Rc::new(RefCell::new($inside))
    };
}

fn interpret_operand_expression(
    operand_expression: &AOperandExpression,
    ds: &DataScope,
    gd: &GlobalData,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    match &operand_expression.value {
        AOperandExpressionValue::Dot { left, right } => {
            let left = interpret_operand_expression(&left, ds, gd)?;
            return left
                .borrow()
                .resolve_aobject(right, Some(left.clone()), ds, gd);
        }
        AOperandExpressionValue::Object(aobject) => ds.resolve_aobject(aobject, gd, ds),
        AOperandExpressionValue::Literal(aliteral) => Ok(Data::from_aliteral(aliteral, gd)),
        AOperandExpressionValue::Create { _type, args } => {
            Data::create_new(_type.clone(), args, ds, gd)
        }
    }
}

fn interpret_termblock(
    termblock: &ATermBlock,
    parent_ds: &DataScope,
    gd: &GlobalData,
) -> Result<BlockExit, RuntimeError> {
    let mut ds = parent_ds.create_child();
    let terms = match termblock {
        ATermBlock::A { terms } => terms,
        ATermBlock::NotYetEvaluated(..) => panic!(),
    };

    for term in terms {
        match term {
            ATerm::Print { ln, value } => {
                let string = match *interpret_operand_expression(value, &ds, gd)?.borrow() {
                    Data::StructObject(StructObject::Root(Root {
                        value: RootValue::String(ref string),
                        ..
                    })) => string.clone(),
                    _ => panic!(),
                };

                match ln {
                    true => println!("{}", string),
                    false => print!("{}", string),
                }
            }
            ATerm::DeclareVar { name, value, .. } => {
                ds.insert_data(name, interpret_operand_expression(value, &ds, gd)?);
            }
            ATerm::Return { value } => {
                return Ok(BlockExit::Explicit(interpret_operand_expression(
                    value, &ds, gd,
                )?))
            }
            ATerm::UpdateVar { value, var } => {
                ds.resolve_aobject(var, gd, &ds)?
                    .swap(&*interpret_operand_expression(value, &ds, gd)?);
            }
            ATerm::If {
                conditional,
                block,
                else_block,
            } => {
                let condition = match *interpret_operand_expression(conditional, &ds, gd)?.borrow()
                {
                    Data::StructObject(StructObject::Root(Root {
                        value: RootValue::Bool(b),
                        ..
                    })) => b,
                    _ => panic!(),
                };

                let exit = match condition {
                    true => interpret_termblock(block, &ds, gd)?,
                    false => interpret_termblock(else_block, &ds, gd)?,
                };

                if let BlockExit::ImplicitNull = exit {
                } else {
                    return Ok(exit);
                }
            }
            ATerm::Call { value } => {
                interpret_operand_expression(value, &ds, gd)?;
            }
            ATerm::Loop {
                counter,
                conditional,
                block,
            } => {
                let counter_var = rc_ref!(Data::StructObject(StructObject::Root(Root {
                    _type: AType::from_astruct(gd.root_types.int_type.clone())
                        .borrow()
                        .to_type_instance(),
                    value: RootValue::Int(0),
                })));

                let mut ds = ds.create_child();
                ds.insert_data(counter, counter_var.clone());

                let mut condition =
                    match *interpret_operand_expression(conditional, &ds, gd)?.borrow() {
                        Data::StructObject(StructObject::Root(Root {
                            value: RootValue::Bool(b),
                            ..
                        })) => b,
                        _ => panic!(),
                    };

                while condition {
                    let exit = interpret_termblock(block, &ds, gd)?;

                    match exit {
                        BlockExit::Break => break,
                        BlockExit::Explicit(..) => return Ok(exit),
                        _ => {
                            if let Data::StructObject(StructObject::Root(Root {
                                ref mut value,
                                ..
                            })) = *counter_var.borrow_mut()
                            {
                                if let RootValue::Int(ref mut int) = value {
                                    *int += 1
                                };
                            }
                        }
                    }

                    condition = match *interpret_operand_expression(conditional, &ds, gd)?.borrow()
                    {
                        Data::StructObject(StructObject::Root(Root {
                            value: RootValue::Bool(b),
                            ..
                        })) => b,
                        _ => panic!(),
                    };
                }
            }
            ATerm::Break => return Ok(BlockExit::Break),
            ATerm::Continue => return Ok(BlockExit::Continue),
        }
    }

    return Ok(BlockExit::ImplicitNull);
}

fn interpret_function(
    func: &AFunc,
    parent_ds: Option<&DataScope>,
    gd: &GlobalData,
    args: &[Rc<RefCell<Data>>],
) -> Result<BlockExit, RuntimeError> {
    let mut ds = match parent_ds {
        Some(parent_ds) => parent_ds.create_child(),
        None => DataScope::new(),
    };
    ds.add_arglist(args, func);

    return match &func.block {
        AFuncBlock::Internal => internal_funcs::interpret_function(func, parent_ds, gd, args),
        AFuncBlock::TermsLang(termblock) => interpret_termblock(&termblock.borrow(), &ds, gd),
        AFuncBlock::InternalArray => {
            internal_funcs::array_internals::interpret_function(func, parent_ds, gd, args)
        }
    };
}

pub fn interpret(program: AProgram, args: Args) -> Result<(), RuntimeError> {
    let mut gd = GlobalData::new(program.root_types);

    for _struct in &program.structs {
        gd.add_struct(_struct.clone());
    }

    for func in &program.functions {
        gd.add_function(func.clone());
    }

    if let Some(main) = gd.functions.get(nm::F_MAIN) {
        let commandline_args = Array(rc_ref!(args
            .args
            .into_iter()
            .map(|x| {
                rc_ref!(Data::StructObject(StructObject::Root(Root {
                    _type: AType::from_astruct(gd.root_types.string_type.clone())
                        .borrow()
                        .to_type_instance(),
                    value: RootValue::String(x),
                })))
            })
            .collect()));

        let args = rc_ref!(Data::ArrayObject(commandline_args));

        interpret_function(&main, None, &gd, &[args])?;
    }

    return Ok(());
}
