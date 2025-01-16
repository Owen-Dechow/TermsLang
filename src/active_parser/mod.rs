pub mod names;

use names as nm;

use crate::{
    errors::{AParserError, FileLocation},
    lexer::tokens::{Operator, Token, TokenType},
    parser::{
        parse_operand_block::{OperandExpression, OperandExpressionValue},
        Call, Object, ObjectType, Program, Term, TermBlock, Type,
    },
};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::zip,
    rc::Rc,
};

pub struct GlobalCounter(u32);
impl GlobalCounter {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&mut self) -> u32 {
        self.0 += 1;
        return self.0;
    }
}

struct GlobalData {
    structs: HashMap<String, Rc<AStruct>>,
    functions: HashMap<String, Rc<AFunc>>,
    not_yet_defined: Vec<Rc<RefCell<AType>>>,

    int_type: Rc<AStruct>,
    bool_type: Rc<AStruct>,
    string_type: Rc<AStruct>,
    float_type: Rc<AStruct>,
    null_type: Rc<AStruct>,
}
impl GlobalData {
    fn new(gc: &mut GlobalCounter) -> Self {
        let mut new = GlobalData {
            structs: HashMap::new(),
            functions: HashMap::new(),
            not_yet_defined: Vec::new(),

            int_type: AStruct::tmp_empty_root().into(),
            bool_type: AStruct::tmp_empty_root().into(),
            string_type: AStruct::tmp_empty_root().into(),
            float_type: AStruct::tmp_empty_root().into(),
            null_type: AStruct::tmp_empty_root().into(),
        };

        new.int_type = new.add_root_struct(
            nm::INT,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_NEW, nm::INT, &[nm::INT]),
                (nm::F_ADD, nm::INT, &[nm::INT]),
                (nm::F_SUB, nm::INT, &[nm::INT]),
                (nm::F_MULT, nm::INT, &[nm::INT]),
                (nm::F_DIV, nm::INT, &[nm::INT]),
                (nm::F_MOD, nm::INT, &[nm::INT]),
                (nm::F_EXP, nm::INT, &[nm::INT]),
                (nm::F_EQ, nm::BOOL, &[nm::INT]),
                (nm::F_GT, nm::BOOL, &[nm::INT]),
                (nm::F_GTEQ, nm::BOOL, &[nm::INT]),
                (nm::F_LT, nm::BOOL, &[nm::INT]),
                (nm::F_LTEQ, nm::BOOL, &[nm::INT]),
            ],
            gc,
        );
        new.null_type = new.add_root_struct(
            nm::NULL,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_NEW, nm::NULL, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_EQ, nm::BOOL, &[nm::NULL]),
            ],
            gc,
        );
        new.float_type = new.add_root_struct(
            nm::FLOAT,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_NEW, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_ADD, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_SUB, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_MULT, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_DIV, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_MOD, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_EXP, nm::FLOAT, &[nm::FLOAT]),
                (nm::F_EQ, nm::BOOL, &[nm::FLOAT]),
                (nm::F_GT, nm::BOOL, &[nm::FLOAT]),
                (nm::F_GTEQ, nm::BOOL, &[nm::FLOAT]),
                (nm::F_LT, nm::BOOL, &[nm::FLOAT]),
                (nm::F_LTEQ, nm::BOOL, &[nm::FLOAT]),
            ],
            gc,
        );
        new.bool_type = new.add_root_struct(
            nm::BOOL,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_NEW, nm::INT, &[]),
                (nm::F_NOT, nm::BOOL, &[]),
                (nm::F_AND, nm::BOOL, &[]),
                (nm::F_OR, nm::BOOL, &[]),
            ],
            gc,
        );
        new.string_type = new.add_root_struct(
            nm::STRING,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_NEW, nm::STRING, &[nm::STRING]),
                (nm::F_LEN, nm::INT, &[]),
                (nm::F_ADD, nm::STRING, &[nm::STRING]),
                (nm::F_MOD, nm::STRING, &[nm::STRING]),
                (nm::F_EQ, nm::BOOL, &[nm::STRING]),
            ],
            gc,
        );

        new.add_root_function(nm::F_READLN, new.string_type.clone(), gc);

        return new;
    }

    fn create_forward_ref(&mut self, name: &str) -> Rc<RefCell<AType>> {
        AType::from_type_nyd(
            &Type::Object {
                object: Object {
                    loc: FileLocation::None,
                    kind: ObjectType::Identity(name.to_owned()),
                    sub: None,
                },
            },
            self,
        )
    }

    fn add_root_struct(
        &mut self,
        name: &str,
        funcs: &[(&str, &str, &[&str])],
        gc: &mut GlobalCounter,
    ) -> Rc<AStruct> {
        let mut a_funcs = HashMap::new();
        for (func_name, func_return, func_args) in funcs {
            let mut args = Vec::new();
            for arg in *func_args {
                let a_arg = AVarDef {
                    name: String::new(),
                    _type: self.create_forward_ref(arg),
                };

                args.push(a_arg);
            }

            let a_func = AFunc {
                name: func_name.to_string(),
                returntype: self.create_forward_ref(&func_return),
                block: AFuncBlock::Internal,
                args,
                loc: FileLocation::None,
                uid: gc.next(),
            };

            a_funcs.insert(func_name.to_string(), a_func.into());
        }

        let a_struct = Rc::new(AStruct {
            name: name.to_string(),
            fields: HashMap::new(),
            methods: a_funcs,
            root: true,
        });

        self.structs.insert(name.to_string(), a_struct.clone());
        return a_struct;
    }

    fn add_root_function(&mut self, name: &str, returntype: Rc<AStruct>, gc: &mut GlobalCounter) {
        self.functions.insert(
            name.to_string(),
            AFunc {
                name: name.to_string(),
                returntype: AType::from_astruct(returntype),
                block: AFuncBlock::Internal,
                args: Vec::new(),
                loc: FileLocation::None,
                uid: gc.next(),
            }
            .into(),
        );
    }

    fn resolve_id(
        &self,
        id: &String,
        loc: &FileLocation,
    ) -> Result<Rc<RefCell<AType>>, AParserError> {
        match self.structs.get(id) {
            Some(some) => Ok(RefCell::new(AType::StructDefRef(some.clone())).into()),
            None => match self.functions.get(id) {
                Some(some) => Ok(RefCell::new(AType::FuncDefRef(some.clone())).into()),
                None => {
                    return Err(AParserError(
                        format!("No object of name {} exists.", id),
                        loc.clone(),
                    ))
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct ARootTypeCollection {
    pub int_type: Rc<AStruct>,
    pub bool_type: Rc<AStruct>,
    pub string_type: Rc<AStruct>,
    pub float_type: Rc<AStruct>,
    pub null_type: Rc<AStruct>,
}

struct DataScope<'a> {
    parent: Option<&'a DataScope<'a>>,
    vars: HashMap<String, Rc<RefCell<AType>>>,
}
impl<'a> DataScope<'a> {
    fn new() -> Self {
        DataScope {
            parent: None,
            vars: HashMap::new(),
        }
    }

    fn child(&'a self) -> Self {
        DataScope {
            parent: Some(self),
            vars: HashMap::new(),
        }
    }

    fn resolve_id(
        &self,
        id: &String,
        gd: &GlobalData,
        loc: &FileLocation,
    ) -> Result<Rc<RefCell<AType>>, AParserError> {
        match self.vars.get(id) {
            Some(some) => Ok(some.clone()),
            None => match &self.parent {
                Some(parent) => parent.resolve_id(id, gd, loc),
                None => gd.resolve_id(id, loc),
            },
        }
    }

    fn resolve_type(
        &self,
        _type: &Type,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<Rc<RefCell<AType>>, AParserError> {
        match _type {
            Type::Array { _type, .. } => {
                let a_type = Rc::try_unwrap(self.resolve_type(&_type, gd, gc)?).unwrap();
                return Ok(RefCell::new(AType::ArrayObject(a_type.into())).into());
            }
            Type::Object { object } => Ok(AObject::from_object(object, self, gd, gc)?._type),
        }
    }

    fn from_func_args(func: &AFunc) -> Self {
        let mut new = DataScope::new();
        for arg in &func.args {
            new.vars
                .insert(arg.name.clone(), arg._type.borrow().to_type_instance());
        }

        return new;
    }

    fn from_func_args_this(func: &AFunc, this: Rc<AStruct>) -> Self {
        let mut new = Self::from_func_args(func);
        new.vars.insert(
            names::THIS.to_string(),
            AType::from_astruct(this).borrow().to_type_instance(),
        );

        return new;
    }
}

#[derive(Debug)]
pub struct AProgram {
    pub structs: Vec<Rc<AStruct>>,
    pub functions: Vec<Rc<AFunc>>,
    pub root_types: ARootTypeCollection,
}

#[derive(Debug)]
pub enum ATermBlock {
    A { terms: Vec<ATerm> },
    NotYetEvaluated(TermBlock),
}

#[derive(Debug)]
pub enum ATerm {
    Print {
        ln: bool,
        value: AOperandExpression,
    },
    DeclareVar {
        name: String,
        vartype: Rc<RefCell<AType>>,
        value: AOperandExpression,
    },
    Return {
        value: AOperandExpression,
    },
    UpdateVar {
        value: AOperandExpression,
        var: AObject,
    },
    If {
        conditional: AOperandExpression,
        block: ATermBlock,
        else_block: ATermBlock,
    },
    Call {
        value: AOperandExpression,
    },
    Loop {
        counter: String,
        conditional: AOperandExpression,
        block: ATermBlock,
    },
    Break,
    Continue,
}

pub enum AType {
    ArrayObject(Rc<RefCell<AType>>),
    StructObject(Rc<AStruct>),
    StructDefRef(Rc<AStruct>),
    FuncDefRef(Rc<AFunc>),
    NotYetDefined(Type, bool),
}
impl AType {
    fn from_type_nyd(value: &Type, gd: &mut GlobalData) -> Rc<RefCell<Self>> {
        let a_type = match value {
            Type::Array { _type, .. } => AType::ArrayObject(AType::from_type_nyd(_type, gd)),
            Type::Object { object } => match &object.kind {
                ObjectType::Identity(id) => match gd.structs.get(id) {
                    Some(_type) => AType::StructDefRef(_type.to_owned()),
                    None => AType::NotYetDefined(value.clone(), false),
                },
                _ => panic!("Should be identity"),
            },
        };

        let rc_atype = Rc::from(RefCell::from(a_type));

        if let AType::NotYetDefined(..) = *rc_atype.borrow() {
            gd.not_yet_defined.push(rc_atype.clone());
        }

        return rc_atype;
    }

    fn from_aliteral(value: &ALiteral, gd: &GlobalData) -> Rc<RefCell<Self>> {
        let a_struct = match value {
            ALiteral::Int(_) => &gd.int_type,
            ALiteral::Float(_) => &gd.float_type,
            ALiteral::String(_) => &gd.string_type,
            ALiteral::Bool(_) => &gd.bool_type,
        }
        .clone();

        return RefCell::new(AType::StructObject(a_struct)).into();
    }

    fn structdefref_is_instance(
        &self,
        inst: &Self,
        loc: &FileLocation,
    ) -> Result<bool, AParserError> {
        match (self, inst) {
            (AType::StructDefRef(defref), AType::StructObject(object)) => {
                Ok(Rc::ptr_eq(defref, object))
            }
            (AType::ArrayObject(arr_type), AType::ArrayObject(object)) => arr_type
                .borrow()
                .structdefref_is_instance(&object.borrow(), loc),
            (AType::ArrayObject(..), AType::StructObject(..)) => Ok(false),
            (AType::StructDefRef(..), AType::StructDefRef(astruct)) => Err(AParserError(
                format!("{:?} is a type definition not an instance", astruct.name),
                loc.clone(),
            )),
            _ => panic!(
                "Bad StructDefRef Is Instance Check:\n - self: {:?}\n - inst: {:?}",
                self, inst
            ),
        }
    }

    fn instance_type_match(&self, inst: &Self) -> bool {
        match (self, inst) {
            (AType::StructObject(rc1), AType::StructObject(rc2)) => Rc::ptr_eq(rc1, rc2),
            (AType::ArrayObject(rc1), AType::ArrayObject(rc2)) => {
                rc1.borrow().instance_type_match(&rc2.borrow())
            }
            (AType::ArrayObject(..), AType::StructObject(..)) => false,
            _ => panic!("{:?} > {:?}", self, inst),
        }
    }

    fn is_nulldef(&self, gd: &GlobalData) -> bool {
        match self {
            AType::StructDefRef(struct_def) => Rc::ptr_eq(&struct_def, &gd.null_type),
            _ => false,
        }
    }

    pub fn to_type_instance(&self) -> Rc<RefCell<Self>> {
        match self {
            AType::StructDefRef(rc) => RefCell::new(AType::StructObject(rc.clone())).into(),
            AType::ArrayObject(arr_type) => {
                RefCell::new(AType::ArrayObject(arr_type.borrow().to_type_instance())).into()
            }
            _ => panic!("{self:?}"),
        }
    }

    fn to_type_instance_nyd(&self, gd: &mut GlobalData) -> Rc<RefCell<Self>> {
        match self {
            AType::StructDefRef(rc) => RefCell::new(AType::StructObject(rc.clone())).into(),
            AType::ArrayObject(arr_type) => {
                RefCell::new(AType::ArrayObject(arr_type.borrow().to_type_instance())).into()
            }
            AType::NotYetDefined(_type, false) => {
                let new = Rc::new(RefCell::new(AType::NotYetDefined(_type.clone(), true)));
                gd.not_yet_defined.push(new.clone());

                return new;
            }
            _ => panic!("{self:?}"),
        }
    }

    fn to_type_defref(&self) -> Rc<RefCell<Self>> {
        match self {
            AType::StructObject(rc) => RefCell::new(AType::StructDefRef(rc.clone())).into(),
            AType::ArrayObject(arr_type) => {
                RefCell::new(AType::ArrayObject(arr_type.borrow().to_type_defref())).into()
            }
            _ => panic!(),
        }
    }

    pub fn from_astruct(astruct: Rc<AStruct>) -> Rc<RefCell<Self>> {
        RefCell::new(AType::StructDefRef(astruct)).into()
    }
}

impl Debug for AType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArrayObject(arg0) => f.debug_tuple(&format!("{:?}[]", arg0.borrow())).finish(),
            Self::StructObject(arg0) => f.debug_tuple(&format!("$({})", arg0.name)).finish(),
            Self::StructDefRef(arg0) => f.debug_tuple(&format!("{}", arg0.name)).finish(),
            Self::FuncDefRef(arg0) => f.debug_tuple(&format!("func({})", arg0.name)).finish(),
            Self::NotYetDefined(arg0, t) => f
                .debug_tuple(&format!("NotYetDefined({:?}, {t})", arg0))
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AObject {
    pub kind: AObjectType,
    pub sub: Option<Box<AObject>>,
    pub _type: Rc<RefCell<AType>>,
    pub loc: FileLocation,
}
impl AObject {
    fn from_object(
        object: &Object,
        ds: &DataScope,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<AObject, AParserError> {
        let (_type, id) = match &object.kind {
            ObjectType::Identity(id) => (ds.resolve_id(id, gd, &object.loc)?, id),
            ObjectType::Call(_) => panic!("Attempt to directly resolve call"),
            ObjectType::Index(_) => panic!("Attempt to directly resolve index"),
        };

        let sub = if let Some(ref sub) = object.sub {
            Some(Box::new(AObject::from_object_sub(
                &sub,
                &_type.borrow(),
                ds,
                gd,
                gc,
            )?))
        } else {
            None
        };

        return Ok(AObject {
            _type,
            sub,
            kind: AObjectType::Identity(id.to_string()),
            loc: object.loc.clone(),
        });
    }

    fn from_object_sub(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<AObject, AParserError> {
        match parent_type {
            AType::ArrayObject(_) => {
                AObject::from_object_sub_array(object, parent_type, ds, gd, gc)
            }
            AType::StructObject(_) => {
                AObject::from_object_sub_struct(object, parent_type, ds, gd, gc)
            }
            AType::StructDefRef(_) => {
                return Err(AParserError(
                    format!("Cannot get field or method on struct definition"),
                    object.loc.clone(),
                ))
            }
            AType::FuncDefRef(_) => {
                AObject::from_object_sub_function(object, parent_type, ds, gd, gc)
            }
            _ => panic!("{:?}", parent_type),
        }
    }

    fn from_object_sub_function(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<AObject, AParserError> {
        let call = match &object.kind {
            ObjectType::Identity(_) => {
                return Err(AParserError(
                    format!("Cannot get field on function definition"),
                    object.loc.clone(),
                ))
            }
            ObjectType::Index(_) => {
                return Err(AParserError(
                    format!("Cannot index function definition"),
                    object.loc.clone(),
                ))
            }
            ObjectType::Call(call) => call,
        };

        let func = match parent_type {
            AType::FuncDefRef(func) => func,
            _ => panic!(),
        };

        if call.args.len() != func.args.len() {
            return Err(AParserError(
                format!(
                    "{} expects {} arguments: {} given.",
                    func.name,
                    func.args.len(),
                    call.args.len(),
                ),
                object.loc.clone(),
            ));
        }

        let mut acall = ACall { args: Vec::new() };
        for (arg, expected_type) in zip(&call.args, &func.args) {
            let a_arg = aparse_operandexpression(arg, ds, gd, gc)?;

            if !expected_type
                ._type
                .borrow()
                .structdefref_is_instance(&a_arg._type.borrow(), &a_arg.loc)?
            {
                return Err(AParserError(
                    format!("Missmatched arg type."),
                    a_arg.loc.clone(),
                ));
            }

            acall.args.push(a_arg);
        }

        let returntype = func.returntype.borrow().to_type_instance();

        let sub = if let Some(sub) = &object.sub {
            Some(Box::new(AObject::from_object_sub(
                sub,
                &returntype.borrow(),
                ds,
                gd,
                gc,
            )?))
        } else {
            None
        };

        Ok(AObject {
            kind: AObjectType::Call(acall),
            sub,
            _type: returntype,
            loc: object.loc.clone(),
        })
    }

    fn from_object_sub_array(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<AObject, AParserError> {
        let arr_type = match &parent_type {
            AType::ArrayObject(rc) => rc.clone(),
            _ => panic!(),
        };

        match &object.kind {
            ObjectType::Identity(id) => {
                let (returntype, args) = match id.as_str() {
                    nm::F_INDEX => (
                        arr_type.borrow().to_type_defref(),
                        vec![AVarDef {
                            name: String::from("idx"),
                            _type: AType::from_astruct(gd.int_type.clone()),
                        }],
                    ),
                    nm::F_APPEND => (
                        AType::from_astruct(gd.null_type.clone()),
                        vec![AVarDef {
                            name: String::from("idx"),
                            _type: arr_type.borrow().to_type_defref(),
                        }],
                    ),
                    nm::F_REMOVE => (
                        arr_type.borrow().to_type_defref(),
                        vec![AVarDef {
                            name: String::from("idx"),
                            _type: AType::from_astruct(gd.int_type.clone()),
                        }],
                    ),
                    nm::F_LEN => (AType::from_astruct(gd.int_type.clone()), vec![]),

                    _ => {
                        return Err(AParserError(
                            format!("{} is not a reconized method of vectors.", id),
                            object.loc.clone(),
                        ))
                    }
                };

                let func = AType::FuncDefRef(
                    AFunc {
                        name: id.to_string(),
                        returntype,
                        block: AFuncBlock::InternalArray,
                        args,
                        loc: FileLocation::None,
                        uid: gc.next(),
                    }
                    .into(),
                );

                let _type = Rc::new(RefCell::new(func));

                return Ok(AObject {
                    kind: AObjectType::Identity(id.clone()),
                    sub: match &object.sub {
                        Some(sub) => Some(Box::new(AObject::from_object_sub_function(
                            &sub,
                            &_type.borrow(),
                            ds,
                            gd,
                            gc,
                        )?)),
                        None => None,
                    },
                    _type,
                    loc: object.loc.clone(),
                });
            }
            ObjectType::Call(_) => Err(AParserError(
                format!("Cannot directly call vector."),
                object.loc.clone(),
            )),
            ObjectType::Index(operand_expression) => {
                let name = nm::F_INDEX.to_string();
                let returntype = match parent_type {
                    AType::ArrayObject(rc) => rc.clone(),
                    _ => panic!(),
                };

                let args = vec![AVarDef {
                    name: String::from("idx"),
                    _type: AType::from_astruct(gd.int_type.clone()),
                }];

                let func = AType::FuncDefRef(
                    AFunc {
                        name: name.clone(),
                        returntype: returntype.borrow().to_type_defref(),
                        block: AFuncBlock::InternalArray,
                        args,
                        loc: FileLocation::None,
                        uid: gc.next(),
                    }
                    .into(),
                );

                let _type = Rc::new(RefCell::new(func));

                let arg = aparse_operandexpression(operand_expression, ds, gd, gc)?;

                let sub = match &object.sub {
                    Some(sub) => Some(Box::new(AObject::from_object_sub(
                        &sub,
                        &returntype.borrow(),
                        ds,
                        gd,
                        gc,
                    )?)),
                    None => None,
                };

                return Ok(AObject {
                    kind: AObjectType::Identity(name),
                    sub: Some(Box::new(AObject {
                        kind: AObjectType::Call(ACall { args: vec![arg] }),
                        sub,
                        _type: returntype,
                        loc: object.loc.clone(),
                    })),
                    _type,
                    loc: object.loc.clone(),
                });
            }
        }
    }

    fn from_object_sub_struct(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
        gc: &mut GlobalCounter,
    ) -> Result<AObject, AParserError> {
        let astruct = match parent_type {
            AType::StructObject(astruct) => astruct,
            _ => panic!(),
        };

        let (kind, _type, ..) = match &object.kind {
            ObjectType::Identity(id) => {
                let (_type, connected_instance_type) = match astruct.fields.get(id) {
                    Some(some) => (some._type.clone(), None),
                    None => match astruct.methods.get(id) {
                        Some(some) => (
                            RefCell::new(AType::FuncDefRef(some.clone())).into(),
                            Some(parent_type),
                        ),
                        None => {
                            return Err(AParserError(
                                format!(
                                    "Struct object {} has no field or method {}.",
                                    astruct.name, id
                                ),
                                object.loc.clone(),
                            ));
                        }
                    },
                };

                (
                    AObjectType::Identity(id.clone()),
                    _type,
                    connected_instance_type,
                )
            }
            ObjectType::Call(..) => {
                return Err(AParserError(
                    format!("Cannot directly call function on struct"),
                    object.loc.clone(),
                ))
            }
            ObjectType::Index(idx) => {
                let func = match astruct.methods.get(nm::F_INDEX) {
                    Some(func) => func,
                    None => {
                        return Err(AParserError(
                            format!("{} has no method {}", astruct.name, nm::F_INDEX),
                            object.loc.clone(),
                        ))
                    }
                };

                let object = AObject::from_object_sub_function(
                    &Object {
                        loc: object.loc.clone(),
                        kind: ObjectType::Call(Call {
                            args: vec![*idx.clone()],
                        }),
                        sub: None,
                    },
                    &AType::FuncDefRef(func.clone()),
                    ds,
                    gd,
                    gc,
                )?;

                (object.kind, object._type, None)
            }
        };

        let sub = if let Some(ref sub) = object.sub {
            Some(Box::new(AObject::from_object_sub(
                &sub,
                &_type.borrow(),
                ds,
                gd,
                gc,
            )?))
        } else {
            None
        };

        return Ok(AObject {
            kind,
            sub,
            _type,
            loc: object.loc.clone(),
        });
    }

    fn bottom_type(&self) -> Rc<RefCell<AType>> {
        match &self.sub {
            Some(some) => some.bottom_type(),
            None => self._type.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AObjectType {
    Identity(String),
    Call(ACall),
}

#[derive(Debug, Clone)]
pub struct ACall {
    pub args: Vec<AOperandExpression>,
}

#[derive(Debug)]
pub struct AVarDef {
    pub name: String,
    pub _type: Rc<RefCell<AType>>,
}

#[derive(Debug)]
pub enum AFuncBlock {
    Internal,
    TermsLang(Rc<RefCell<ATermBlock>>),
    InternalArray,
}

#[derive(Debug)]
pub struct AFunc {
    pub name: String,
    pub returntype: Rc<RefCell<AType>>,
    pub block: AFuncBlock,
    pub args: Vec<AVarDef>,
    pub loc: FileLocation,
    pub uid: u32,
}

#[derive(Debug, Clone)]
pub enum ALiteral {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}
impl ALiteral {
    fn from_token_literal(token: &Token) -> Self {
        match &token.0 {
            TokenType::Int(val) => ALiteral::Int(*val),
            TokenType::Float(val) => ALiteral::Float(*val),
            TokenType::String(val) => ALiteral::String(val.to_owned()),
            TokenType::Bool(val) => ALiteral::Bool(*val),
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AOperandExpression {
    _type: Rc<RefCell<AType>>,
    pub loc: FileLocation,
    pub value: AOperandExpressionValue,
}

#[derive(Debug, Clone)]
pub enum AOperandExpressionValue {
    Dot {
        left: Box<AOperandExpression>,
        right: AObject,
    },
    Object(AObject),
    Literal(ALiteral),
    Create {
        _type: Rc<RefCell<AType>>,
        args: Vec<AOperandExpression>,
    },
}

#[derive(Debug)]
pub struct AStruct {
    pub name: String,
    pub fields: HashMap<String, AVarDef>,
    pub methods: HashMap<String, Rc<AFunc>>,
    pub root: bool,
}
impl AStruct {
    fn tmp_empty_root() -> Self {
        Self {
            name: String::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            root: true,
        }
    }

    fn astruct_type_object_match(astruct: &Rc<Self>, inst: &AType) -> bool {
        match inst {
            AType::StructObject(rc) => Rc::ptr_eq(astruct, rc),
            _ => false,
        }
    }
}

#[derive(Clone)]
struct ReturnOpts {
    expected_type: Rc<RefCell<AType>>,
    loop_returns: bool,
    require_explicit: bool,
}
impl ReturnOpts {
    fn loop_opts(&self) -> Self {
        let mut new = self.clone();
        new.loop_returns = true;
        return new;
    }

    fn requirement_free_opts(&self) -> Self {
        let mut new = self.clone();
        new.require_explicit = false;
        return new;
    }
}

fn aparse_operandexpression(
    operand_expression: &OperandExpression,
    ds: &DataScope,
    gd: &GlobalData,
    gc: &mut GlobalCounter,
) -> Result<AOperandExpression, AParserError> {
    match &operand_expression.0 {
        OperandExpressionValue::Unary { operand, val } => {
            let func = match &operand.0 {
                TokenType::Operator(Operator::Not) => nm::F_NOT,
                _ => panic!(),
            };

            let left = aparse_operandexpression(&val, ds, gd, gc)?;
            let object = Object {
                loc: operand.1.clone(),
                kind: ObjectType::Identity(func.to_owned()),
                sub: Some(Box::new(Object {
                    loc: operand.1.clone(),
                    kind: ObjectType::Call(Call { args: Vec::new() }),
                    sub: None,
                })),
            };

            let right = AObject::from_object_sub(&object, &left._type.borrow(), ds, gd, gc)?;

            Ok(AOperandExpression {
                _type: right.bottom_type().clone(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right,
                },
                loc: operand.1.clone(),
            })
        }
        OperandExpressionValue::Binary {
            operand,
            left,
            right,
        } => {
            let func = match operand.0 {
                TokenType::Operator(Operator::Add) => nm::F_ADD,
                TokenType::Operator(Operator::Subtract) => nm::F_SUB,
                TokenType::Operator(Operator::Multiply) => nm::F_MULT,
                TokenType::Operator(Operator::Divide) => nm::F_DIV,
                TokenType::Operator(Operator::Modulo) => nm::F_MOD,
                TokenType::Operator(Operator::Exponent) => nm::F_EXP,
                TokenType::Operator(Operator::Equal) => nm::F_EQ,
                TokenType::Operator(Operator::Greater) => nm::F_GT,
                TokenType::Operator(Operator::GreaterOrEqual) => nm::F_GTEQ,
                TokenType::Operator(Operator::Less) => nm::F_LT,
                TokenType::Operator(Operator::LessOrEqual) => nm::F_LTEQ,
                TokenType::Operator(Operator::And) => nm::F_AND,
                TokenType::Operator(Operator::Or) => nm::F_OR,
                _ => panic!(),
            };
            let left = aparse_operandexpression(&left, ds, gd, gc)?;
            let object = Object {
                loc: operand.1.clone(),
                kind: ObjectType::Identity(func.to_owned()),
                sub: Some(Box::new(Object {
                    loc: operand.1.clone(),
                    kind: ObjectType::Call(Call {
                        args: vec![*right.clone()],
                    }),
                    sub: None,
                })),
            };

            let right = AObject::from_object_sub(&object, &left._type.borrow(), ds, gd, gc)?;

            Ok(AOperandExpression {
                _type: right.bottom_type(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right,
                },
                loc: object.loc.clone(),
            })
        }
        OperandExpressionValue::Dot { left, right } => {
            let left = aparse_operandexpression(&left, ds, gd, gc)?;
            let loc = left.loc.clone();
            let right = AObject::from_object_sub(&right, &left._type.borrow(), ds, gd, gc)?;
            Ok(AOperandExpression {
                _type: right.bottom_type(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right,
                },
                loc,
            })
        }
        OperandExpressionValue::Literal(literal) => {
            let a_literal = ALiteral::from_token_literal(&literal);
            let a_type = AType::from_aliteral(&a_literal, gd);
            return Ok(AOperandExpression {
                _type: a_type,
                value: AOperandExpressionValue::Literal(a_literal),
                loc: literal.1.clone(),
            });
        }
        OperandExpressionValue::Object(obj) => {
            let a_object = AObject::from_object(&obj, ds, gd, gc)?;
            let loc = a_object.loc.clone();

            return Ok(AOperandExpression {
                _type: a_object.bottom_type(),
                value: AOperandExpressionValue::Object(a_object),
                loc,
            });
        }
        OperandExpressionValue::Create(create) => {
            let _type = ds.resolve_type(&create.kind, gd, gc)?;
            let new_method = match *_type.borrow() {
                AType::StructDefRef(ref rc) => rc.methods.get(nm::F_NEW).cloned(),
                AType::ArrayObject(..) => None,
                _ => panic!(),
            };

            let args = if let Some(new_method) = new_method {
                let mut args = Vec::new();

                for (arg, argdef) in zip(&create.args.args, &new_method.args) {
                    let arg = aparse_operandexpression(&arg, ds, gd, gc)?;

                    if !argdef
                        ._type
                        .borrow()
                        .structdefref_is_instance(&arg._type.borrow(), &arg.loc)?
                    {
                        return Err(AParserError(
                            format!("Arg to {} was incorrect type.", nm::F_NEW),
                            arg.loc.clone(),
                        ));
                    }

                    args.push(arg);
                }

                if new_method.args.len() != create.args.args.len() {
                    return Err(AParserError(
                        format!("Invalid number of args to {}.", nm::F_NEW),
                        operand_expression.1.clone(),
                    ));
                }

                args
            } else {
                if create.args.args.len() > 0 {
                    return Err(AParserError(
                        format!(
                            "{:?} has no explicit {} function, therefore $() should not take any arguments.",
                            _type.borrow(),
                            nm::F_NEW
                        ),
                        create.args.args[0].1.clone(),
                    ));
                }

                Vec::new()
            };

            return Ok(AOperandExpression {
                _type: _type.borrow().to_type_instance(),
                value: AOperandExpressionValue::Create {
                    _type: _type.clone(),
                    args,
                },
                loc: operand_expression.1.clone(),
            });
        }
    }
}

fn aparse_termblock(
    block: &TermBlock,
    parent_ds: &DataScope,
    gd: &GlobalData,
    gc: &mut GlobalCounter,
    return_opts: &ReturnOpts,
    loc: &FileLocation,
) -> Result<ATermBlock, AParserError> {
    let mut a_terms = Vec::new();
    let mut ds = parent_ds.child();
    let num_terms = block.terms.len();

    for (term_idx, term) in block.terms.iter().enumerate() {
        let a_term = match term {
            Term::Print { ln, operand_block } => {
                let value = aparse_operandexpression(operand_block, &ds, gd, gc)?;

                if !AType::from_astruct(gd.string_type.clone())
                    .borrow()
                    .structdefref_is_instance(&value._type.borrow(), &value.loc)?
                {
                    return Err(AParserError(
                        "Cannot print non string objects.".to_string(),
                        value.loc.clone(),
                    ));
                }

                ATerm::Print { ln: *ln, value }
            }
            Term::DeclareVar {
                name,
                vartype,
                value,
            } => {
                let a_type = ds.resolve_type(vartype, gd, gc)?;
                let a_value = aparse_operandexpression(value, &ds, gd, gc)?;

                if !a_type
                    .borrow()
                    .structdefref_is_instance(&a_value._type.borrow(), &a_value.loc)?
                {
                    return Err(AParserError(
                        format!("Value type does not match var type."),
                        a_value.loc.clone(),
                    ));
                }

                let vartype = a_type.borrow().to_type_instance();
                ds.vars.insert(name.clone(), vartype.clone());

                ATerm::DeclareVar {
                    name: name.to_owned(),
                    vartype,
                    value: a_value,
                }
            }
            Term::Return { value } => {
                let value = aparse_operandexpression(value, &ds, gd, gc)?;
                if !return_opts
                    .expected_type
                    .borrow()
                    .structdefref_is_instance(&value._type.borrow(), &value.loc)?
                {
                    if !value
                        ._type
                        .borrow()
                        .to_type_defref()
                        .borrow()
                        .is_nulldef(gd)
                    {
                        return Err(AParserError(
                            format!("Incorrect type retuned from function."),
                            value.loc.clone(),
                        ));
                    }
                }

                if term_idx != num_terms - 1 {
                    return Err(AParserError(
                        format!("Return must be last term in block."),
                        value.loc.clone(),
                    ));
                }

                ATerm::Return { value }
            }
            Term::UpdateVar {
                var,
                set_operator,
                value,
            } => {
                let operand_expression = |op: Operator, left: Object, right: OperandExpression| {
                    let loc = left.loc.clone();
                    OperandExpression(
                        OperandExpressionValue::Binary {
                            operand: Token(TokenType::Operator(op), left.loc.clone()),
                            left: Box::new(OperandExpression(
                                OperandExpressionValue::Object(left),
                                loc.clone(),
                            )),
                            right: Box::new(right),
                        },
                        loc,
                    )
                };

                let operand_expression = match set_operator {
                    Operator::Set => value.clone(),
                    Operator::SetAdd => {
                        operand_expression(Operator::Add, var.clone(), value.clone())
                    }
                    Operator::SetSubtract => {
                        operand_expression(Operator::Subtract, var.clone(), value.clone())
                    }
                    Operator::SetMultiply => {
                        operand_expression(Operator::Multiply, var.clone(), value.clone())
                    }
                    Operator::SetDivide => {
                        operand_expression(Operator::Divide, var.clone(), value.clone())
                    }
                    Operator::SetModulo => {
                        operand_expression(Operator::Modulo, var.clone(), value.clone())
                    }
                    Operator::SetExponent => {
                        operand_expression(Operator::Exponent, var.clone(), value.clone())
                    }
                    _ => panic!(),
                };

                let value = aparse_operandexpression(&operand_expression, &ds, gd, gc)?;
                let var = AObject::from_object(var, &ds, gd, gc)?;

                if !AType::instance_type_match(&var.bottom_type().borrow(), &value._type.borrow()) {
                    return Err(AParserError(
                        format!("Missmatched types (2)"),
                        var.loc.clone(),
                    ));
                }

                ATerm::UpdateVar { value, var }
            }
            Term::If {
                conditional,
                block,
                else_block,
            } => {
                let conditional = aparse_operandexpression(conditional, &ds, gd, gc)?;
                if !AStruct::astruct_type_object_match(&gd.bool_type, &conditional._type.borrow()) {
                    return Err(AParserError(
                        format!(
                            "Conditional must be of type bool, found type {:?}",
                            &conditional._type.borrow()
                        ),
                        conditional.loc.clone(),
                    ));
                }

                let return_opts = if term_idx == num_terms - 1 {
                    return_opts.clone()
                } else {
                    return_opts.requirement_free_opts()
                };

                let block = aparse_termblock(block, &ds, gd, gc, &return_opts, loc)?;
                let else_block = aparse_termblock(else_block, &ds, gd, gc, &return_opts, loc)?;

                ATerm::If {
                    conditional,
                    block,
                    else_block,
                }
            }
            Term::Loop {
                counter,
                conditional,
                block,
            } => {
                let mut ds = ds.child();

                ds.vars.insert(
                    counter.to_string(),
                    AType::from_astruct(gd.int_type.clone())
                        .borrow()
                        .to_type_instance(),
                );

                let conditional = aparse_operandexpression(conditional, &ds, gd, gc)?;

                let return_opts = return_opts.requirement_free_opts().loop_opts();

                let block = aparse_termblock(block, &ds, gd, gc, &return_opts, loc)?;

                ATerm::Loop {
                    counter: counter.to_string(),
                    conditional,
                    block,
                }
            }
            Term::Break(loc) => {
                if return_opts.loop_returns {
                    ATerm::Break
                } else {
                    return Err(AParserError(
                        format!("Cannot break from outside loop."),
                        loc.clone(),
                    ));
                }
            }
            Term::Continue(loc) => {
                if return_opts.loop_returns {
                    ATerm::Continue
                } else {
                    return Err(AParserError(
                        format!("Cannot continue from outside loop."),
                        loc.clone(),
                    ));
                }
            }
            Term::Call { value } => {
                let value = aparse_operandexpression(value, &ds, gd, gc)?;
                ATerm::Call { value }
            }
        };

        a_terms.push(a_term);
    }

    if return_opts.require_explicit {
        match block.terms.last() {
            Some(Term::Return { .. } | Term::If { .. }) => {}
            _ => {
                if !return_opts.expected_type.borrow().is_nulldef(gd) {
                    return Err(AParserError(
                        format!("Not all paths return correct type"),
                        loc.clone(),
                    ));
                }
            }
        }
    }

    return Ok(ATermBlock::A { terms: a_terms });
}

pub fn aparse(program: &Program) -> Result<AProgram, AParserError> {
    let mut names = HashSet::new();
    let mut gc = GlobalCounter::new();
    let mut gd = GlobalData::new(&mut gc);
    let mut structs = Vec::new();
    let mut functions = Vec::new();

    for _struct in &program.structs {
        if names.contains(&_struct.name) {
            return Err(AParserError(
                format!("Global object {} has multiple definitions.", _struct.name),
                _struct.loc.clone(),
            ));
        } else {
            names.insert(&_struct.name);
        }

        let name = _struct.name.clone();
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();

        for prop in &_struct.properties {
            let name = prop.identity.clone();
            let _type = AType::from_type_nyd(&prop.argtype, &mut gd)
                .borrow()
                .to_type_instance_nyd(&mut gd);

            let field = AVarDef { name, _type };
            fields.insert(field.name.to_owned(), field);
        }

        for method in &_struct.methods {
            let returntype = AType::from_type_nyd(&method.returntype, &mut gd);
            let name = method.name.clone();
            let loc = method.loc.clone();

            let mut args = Vec::new();
            for arg in &method.args {
                let name = arg.identity.to_owned();
                let _type = AType::from_type_nyd(&arg.argtype, &mut gd);
                let a_arg = AVarDef { name, _type };
                args.push(a_arg);
            }

            let block = AFuncBlock::TermsLang(
                RefCell::new(ATermBlock::NotYetEvaluated(method.block.clone())).into(),
            );

            let a_method = AFunc {
                name,
                returntype,
                block,
                args,
                loc,
                uid: gc.next(),
            };

            methods.insert(a_method.name.to_owned(), a_method.into());
        }

        let a_struct = Rc::new(AStruct {
            name,
            fields,
            methods,
            root: false,
        });

        gd.structs.insert(_struct.name.clone(), a_struct.clone());
        structs.push(a_struct);
    }

    for func in &program.functions {
        if names.contains(&func.name) {
            return Err(AParserError(
                format!("Global object {} has multiple definitions.", func.name),
                func.loc.clone(),
            ));
        } else {
            names.insert(&func.name);
        }

        let returntype = AType::from_type_nyd(&func.returntype, &mut gd);
        let name = func.name.clone();
        let loc = func.loc.clone();

        let mut args = Vec::new();
        for arg in &func.args {
            let name = arg.identity.to_owned();
            let _type = AType::from_type_nyd(&arg.argtype, &mut gd);
            let a_arg = AVarDef { name, _type };
            args.push(a_arg);
        }

        let block = AFuncBlock::TermsLang(
            RefCell::new(ATermBlock::NotYetEvaluated(func.block.clone())).into(),
        );

        let a_func = Rc::new(AFunc {
            name,
            returntype,
            block,
            args,
            loc,
            uid: gc.next(),
        });

        gd.functions.insert(func.name.clone(), a_func.clone());
        functions.push(a_func);
    }

    // Fix undefined refs
    for undefined in gd.not_yet_defined.clone() {
        let mut a_type_op = None;
        if let AType::NotYetDefined(ref _type, instance) = *undefined.borrow() {
            let a_type = AType::from_type_nyd(_type, &mut gd);

            match *a_type.borrow() {
                AType::NotYetDefined(..) => {
                    return Err(AParserError(
                        format!("Could not find type"),
                        _type.get_location().clone(),
                    ))
                }
                _ => gd.not_yet_defined.remove(0),
            };

            a_type_op = match instance {
                false => Some(a_type),
                true => Some(a_type.borrow().to_type_instance()),
            };
        }

        if let Some(a_type) = a_type_op {
            undefined.swap(&a_type);
        }
    }

    // Fix unfinished functions
    for (_name, func) in &gd.functions {
        let mut new_a_termblock = None;
        if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
            if let ATermBlock::NotYetEvaluated(ref block) = *a_termblock.borrow() {
                let return_specs = ReturnOpts {
                    expected_type: func.returntype.clone(),
                    loop_returns: false,
                    require_explicit: true,
                };
                new_a_termblock = Some(aparse_termblock(
                    block,
                    &DataScope::from_func_args(func),
                    &gd,
                    &mut gc,
                    &return_specs,
                    &func.loc,
                )?);
            }
        }

        if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
            if let Some(some) = new_a_termblock {
                a_termblock.replace(some);
            }
        }
    }

    // Fix unfinished methods
    for (_name, _struct) in &gd.structs {
        for (_name, func) in &_struct.methods {
            let mut new_a_termblock = None;
            if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
                if let ATermBlock::NotYetEvaluated(ref block) = *a_termblock.borrow() {
                    let return_specs = ReturnOpts {
                        expected_type: func.returntype.clone(),
                        loop_returns: false,
                        require_explicit: true,
                    };
                    new_a_termblock = Some(aparse_termblock(
                        block,
                        &DataScope::from_func_args_this(&func, _struct.clone()),
                        &gd,
                        &mut gc,
                        &return_specs,
                        &func.loc,
                    )?);
                }
            }

            if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
                if let Some(some) = new_a_termblock {
                    a_termblock.replace(some);
                }
            }
        }
    }

    let a_program = AProgram {
        structs,
        functions,
        root_types: ARootTypeCollection {
            int_type: gd.int_type,
            bool_type: gd.bool_type,
            string_type: gd.string_type,
            float_type: gd.float_type,
            null_type: gd.null_type,
        },
    };

    Ok(a_program)
}
