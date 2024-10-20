mod names;

use names as nm;

use crate::{
    errors::{AParserError, FileLocation},
    lexer::tokens::{Operator, Token, TokenType},
    parser::{
        parse_operand_block::OperandExpression, Call, Object, ObjectType, Program, Term, TermBlock,
        Type,
    },
};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::zip,
    rc::Rc,
};

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
    fn new() -> Self {
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
        );
        new.null_type = new.add_root_struct(
            nm::NULL,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_NEW, nm::NULL, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_EQ, nm::NULL, &[nm::NULL]),
            ],
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
        );
        new.bool_type = new.add_root_struct(
            nm::BOOL,
            &[
                (nm::F_STRING, nm::STRING, &[]),
                (nm::F_INT, nm::INT, &[]),
                (nm::F_FLOAT, nm::FLOAT, &[]),
                (nm::F_BOOL, nm::BOOL, &[]),
                (nm::F_NEW, nm::INT, &[nm::BOOL]),
                (nm::F_NOT, nm::INT, &[nm::BOOL]),
                (nm::F_AND, nm::INT, &[nm::BOOL]),
                (nm::F_OR, nm::INT, &[nm::BOOL]),
            ],
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
                (nm::F_MULT, nm::STRING, &[nm::INT]),
                (nm::F_MOD, nm::STRING, &[nm::STRING]),
            ],
        );

        new.add_root_function(nm::F_READLN, new.string_type.clone());

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

    fn add_root_struct(&mut self, name: &str, funcs: &[(&str, &str, &[&str])]) -> Rc<AStruct> {
        let mut a_funcs = HashMap::new();
        for func in funcs {
            let mut args = Vec::new();
            for arg in func.2 {
                let a_arg = AVarDef {
                    name: String::new(),
                    _type: self.create_forward_ref(arg),
                };

                args.push(a_arg);
            }

            let a_func = AFunc {
                name: func.0.to_string(),
                returntype: self.create_forward_ref(func.1),
                block: AFuncBlock::Internal,
                args,
            };

            a_funcs.insert(func.0.to_string(), a_func.into());
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

    fn add_root_function(&mut self, name: &str, returntype: Rc<AStruct>) {
        self.functions.insert(
            name.to_string(),
            AFunc {
                name: name.to_string(),
                returntype: AType::from_astruct(returntype),
                block: AFuncBlock::Internal,
                args: Vec::new(),
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
    ) -> Result<Rc<RefCell<AType>>, AParserError> {
        match _type {
            Type::Array { _type, .. } => {
                let a_type = Rc::try_unwrap(self.resolve_type(&_type, gd)?).unwrap();
                return Ok(RefCell::new(AType::ArrayObject(a_type.into())).into());
            }
            Type::Object { object } => Ok(AObject::from_object(object, self, gd)?._type),
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
    NotYetDefined(Type),
}
impl AType {
    fn from_type_nyd(value: &Type, gd: &mut GlobalData) -> Rc<RefCell<Self>> {
        let a_type = match value {
            Type::Array { _type, .. } => AType::ArrayObject(AType::from_type_nyd(_type, gd)),
            Type::Object { object } => match &object.kind {
                ObjectType::Identity(id) => match gd.structs.get(id) {
                    Some(_type) => AType::StructDefRef(_type.to_owned()),
                    None => AType::NotYetDefined(value.clone()),
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

    fn structdefref_is_instance(&self, inst: &Self) -> bool {
        match (self, inst) {
            (AType::StructDefRef(defref), AType::StructObject(object)) => {
                Rc::ptr_eq(defref, object)
            }
            (AType::ArrayObject(arr_type), AType::ArrayObject(object)) => {
                arr_type.borrow().structdefref_is_instance(&object.borrow())
            }
            _ => panic!(
                "Bad StructDefRef Is Instance Check:\n - self: {:?}\n - inst: {:?}",
                self, inst
            ),
        }
    }

    fn instance_type_match(&self, inst: &Self) -> bool {
        match (self, inst) {
            (AType::StructObject(rc1), AType::StructObject(rc2)) => Rc::ptr_eq(rc1, rc2),
            _ => panic!(),
        }
    }

    fn is_nulldef(&self, gd: &GlobalData) -> bool {
        match self {
            AType::StructDefRef(struct_def) => Rc::ptr_eq(&struct_def, &gd.null_type),
            _ => false,
        }
    }

    fn to_type_instance(&self) -> Rc<RefCell<Self>> {
        match self {
            AType::StructDefRef(rc) => RefCell::new(AType::StructObject(rc.clone())).into(),
            AType::ArrayObject(arr_type) => {
                RefCell::new(AType::ArrayObject(arr_type.borrow().to_type_instance())).into()
            }
            _ => panic!(),
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

    fn from_astruct(astruct: Rc<AStruct>) -> Rc<RefCell<Self>> {
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
            Self::NotYetDefined(..) => f.debug_tuple("NotYetDefined").finish(),
        }
    }
}

#[derive(Debug)]
struct AObject {
    kind: AObjectType,
    sub: Option<Box<AObject>>,
    _type: Rc<RefCell<AType>>,
    loc: FileLocation,
}
impl AObject {
    fn from_object(
        object: &Object,
        ds: &DataScope,
        gd: &GlobalData,
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
                None,
                ds,
                gd,
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
        connected_instance_type: Option<&AType>,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<AObject, AParserError> {
        match parent_type {
            AType::ArrayObject(_) => AObject::from_object_sub_array(object, parent_type, ds, gd),
            AType::StructObject(_) => AObject::from_object_sub_struct(object, parent_type, ds, gd),
            AType::StructDefRef(_) => {
                return Err(AParserError(
                    format!("Cannot get field or method on struct definition"),
                    object.loc.clone(),
                ))
            }
            AType::FuncDefRef(_) => AObject::from_object_sub_function(
                object,
                parent_type,
                connected_instance_type,
                ds,
                gd,
            ),
            _ => panic!(),
        }
    }

    fn from_object_sub_function(
        object: &Object,
        parent_type: &AType,
        connected_instance_type: Option<&AType>,
        ds: &DataScope,
        gd: &GlobalData,
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
            let a_arg = aparse_operandexpression(arg, ds, gd)?;

            if !expected_type
                ._type
                .borrow()
                .structdefref_is_instance(&a_arg._type.borrow())
            {
                return Err(AParserError(
                    format!("Missmatched Types (3)"),
                    FileLocation::None,
                ));
            }

            acall.args.push(a_arg);
        }

        let sub = if let Some(sub) = &object.sub {
            Some(Box::new(AObject::from_object_sub(
                sub,
                &func.returntype.borrow().to_type_instance().borrow(),
                None,
                ds,
                gd,
            )?))
        } else {
            None
        };

        Ok(AObject {
            kind: AObjectType::Call(acall),
            sub,
            _type: func.returntype.borrow().to_type_instance(),
            loc: object.loc.clone(),
        })
    }

    fn from_object_sub_array(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<AObject, AParserError> {
        let arr_type = match &parent_type {
            AType::ArrayObject(rc) => rc.clone(),
            _ => panic!(),
        };

        match &object.kind {
            ObjectType::Identity(id) => {
                let (returntype, args) = match id.as_str() {
                    nm::F_INDEX => (
                        arr_type,
                        vec![AVarDef {
                            name: String::from("idx"),
                            _type: AType::from_astruct(gd.int_type.clone()),
                        }],
                    ),
                    nm::F_APPEND => (
                        AType::from_astruct(gd.null_type.clone()),
                        vec![AVarDef {
                            name: String::from("idx"),
                            _type: arr_type,
                        }],
                    ),
                    nm::F_REMOVE => (
                        AType::from_astruct(gd.null_type.clone()),
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
                        returntype: returntype.borrow().to_type_defref(),
                        block: AFuncBlock::InternalArray,
                        args,
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
                            Some(parent_type),
                            ds,
                            gd,
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
                    }
                    .into(),
                );

                let _type = Rc::new(RefCell::new(func));

                let arg = aparse_operandexpression(operand_expression, ds, gd)?;

                let sub = match &object.sub {
                    Some(sub) => Some(Box::new(AObject::from_object_sub_function(
                        &sub,
                        &_type.borrow(),
                        Some(parent_type),
                        ds,
                        gd,
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
    ) -> Result<AObject, AParserError> {
        let astruct = match parent_type {
            AType::StructObject(astruct) => astruct,
            _ => panic!(),
        };

        let (kind, _type, connected_instance_type) = match &object.kind {
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
            ObjectType::Call(call) => {
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
                    None,
                    ds,
                    gd,
                )?;

                (object.kind, object._type, None)
            }
        };

        let sub = if let Some(ref sub) = object.sub {
            Some(Box::new(AObject::from_object_sub(
                &sub,
                &_type.borrow(),
                None,
                ds,
                gd,
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

#[derive(Debug)]
enum AObjectType {
    Identity(String),
    Call(ACall),
}

#[derive(Debug)]
struct ACall {
    args: Vec<AOperandExpression>,
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
}

#[derive(Debug)]
enum ALiteral {
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

#[derive(Debug)]
pub struct AOperandExpression {
    _type: Rc<RefCell<AType>>,
    value: AOperandExpressionValue,
}

#[derive(Debug)]
enum AOperandExpressionValue {
    Dot {
        left: Box<AOperandExpression>,
        right: Box<AOperandExpression>,
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
    expected_type: Option<Rc<RefCell<AType>>>,
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
        new.loop_returns = true;
        return new;
    }
}

fn aparse_operandexpression(
    operand_expression: &OperandExpression,
    ds: &DataScope,
    gd: &GlobalData,
) -> Result<AOperandExpression, AParserError> {
    match operand_expression {
        OperandExpression::Unary { operand, val } => {
            let func = match &operand.0 {
                TokenType::Operator(Operator::Not) => nm::F_NOT,
                _ => panic!(),
            };

            let left = aparse_operandexpression(&val, ds, gd)?;
            let object = Object {
                loc: operand.1.clone(),
                kind: ObjectType::Identity(func.to_owned()),
                sub: Some(Box::new(Object {
                    loc: operand.1.clone(),
                    kind: ObjectType::Call(Call { args: Vec::new() }),
                    sub: None,
                })),
            };

            let right = AObject::from_object_sub(&object, &left._type.borrow(), None, ds, gd)?;

            Ok(AOperandExpression {
                _type: right._type.clone(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right: Box::new(AOperandExpression {
                        _type: right._type.clone(),
                        value: AOperandExpressionValue::Object(right),
                    }),
                },
            })
        }
        OperandExpression::Binary {
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
            let left = aparse_operandexpression(&left, ds, gd)?;
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

            let right = AObject::from_object_sub(&object, &left._type.borrow(), None, ds, gd)?;

            Ok(AOperandExpression {
                _type: right.bottom_type(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right: Box::new(AOperandExpression {
                        _type: right._type.clone(),
                        value: AOperandExpressionValue::Object(right),
                    }),
                },
            })
        }
        OperandExpression::Dot { left, right } => {
            let left = aparse_operandexpression(&left, ds, gd)?;
            let right = AObject::from_object_sub(&right, &left._type.borrow(), None, ds, gd)?;
            Ok(AOperandExpression {
                _type: right.bottom_type(),
                value: AOperandExpressionValue::Dot {
                    left: Box::new(left),
                    right: Box::new(AOperandExpression {
                        _type: right._type.clone(),
                        value: AOperandExpressionValue::Object(right),
                    }),
                },
            })
        }
        OperandExpression::Literal(literal) => {
            let a_literal = ALiteral::from_token_literal(literal);
            let a_type = AType::from_aliteral(&a_literal, gd);
            return Ok(AOperandExpression {
                _type: a_type,
                value: AOperandExpressionValue::Literal(a_literal),
            });
        }
        OperandExpression::Object(obj) => {
            let a_object = AObject::from_object(obj, ds, gd)?;
            return Ok(AOperandExpression {
                _type: a_object.bottom_type(),
                value: AOperandExpressionValue::Object(a_object),
            });
        }
        OperandExpression::Create(create) => {
            let _type = ds.resolve_type(&create.kind, gd)?;
            let new_method = match *_type.borrow() {
                AType::StructDefRef(ref rc) => rc.methods.contains_key(nm::F_NEW),
                AType::ArrayObject(_) => false,
                _ => panic!(),
            };

            let args = if new_method {
                let mut args = Vec::new();
                for arg in &create.args.args {
                    args.push(aparse_operandexpression(&arg, ds, gd)?);
                }

                args
            } else {
                if create.args.args.len() > 0 {
                    return Err(AParserError(
                        format!("{:?} has no explicit {} function, therefore $() should not take any arguments.", _type.borrow(), nm::F_NEW),
                        FileLocation::None,
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
            });
        }
    }
}

fn aparse_termblock(
    block: &TermBlock,
    parent_ds: &DataScope,
    gd: &GlobalData,
    return_opts: &ReturnOpts,
) -> Result<ATermBlock, AParserError> {
    let mut a_terms = Vec::new();
    let mut ds = parent_ds.child();
    let num_terms = block.terms.len();

    for (term_idx, term) in block.terms.iter().enumerate() {
        let a_term = match term {
            Term::Print { ln, operand_block } => ATerm::Print {
                ln: *ln,
                value: aparse_operandexpression(operand_block, &ds, gd)?,
            },
            Term::DeclareVar {
                name,
                vartype,
                value,
            } => {
                let a_type = ds.resolve_type(vartype, gd)?;
                let a_value = aparse_operandexpression(value, &ds, gd)?;

                if !a_type
                    .borrow()
                    .structdefref_is_instance(&a_value._type.borrow())
                {
                    return Err(AParserError(
                        format!("Missmatched types (0)"),
                        FileLocation::None,
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
                let value = aparse_operandexpression(value, &ds, gd)?;
                if let Some(expected_type) = &return_opts.expected_type {
                    if !expected_type
                        .borrow()
                        .structdefref_is_instance(&value._type.borrow())
                    {
                        return Err(AParserError(
                            format!("Missmatched types (1)"),
                            FileLocation::None,
                        ));
                    }
                }

                if term_idx != num_terms - 1 {
                    return Err(AParserError(
                        format!("Return must be last term in block."),
                        FileLocation::None,
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
                    OperandExpression::Binary {
                        operand: Token(TokenType::Operator(op), FileLocation::None),
                        left: Box::new(OperandExpression::Object(left)),
                        right: Box::new(right),
                    }
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

                let value = aparse_operandexpression(&operand_expression, &ds, gd)?;
                let var = AObject::from_object(var, &ds, gd)?;

                if !AType::instance_type_match(&var._type.borrow(), &value._type.borrow()) {
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
                let conditional = aparse_operandexpression(conditional, &ds, gd)?;
                if !AStruct::astruct_type_object_match(&gd.bool_type, &conditional._type.borrow()) {
                    return Err(AParserError(
                        format!(
                            "Conditional must be of type bool, found type {:?}",
                            &conditional._type.borrow()
                        ),
                        FileLocation::None,
                    ));
                }

                let return_opts = if term_idx == num_terms - 1 {
                    return_opts.clone()
                } else {
                    return_opts.requirement_free_opts()
                };

                let block = aparse_termblock(block, &ds, gd, &return_opts)?;
                let else_block = aparse_termblock(else_block, &ds, gd, &return_opts)?;

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

                let conditional = aparse_operandexpression(conditional, &ds, gd)?;

                let return_opts = return_opts.requirement_free_opts().loop_opts();

                let block = aparse_termblock(block, &ds, gd, &return_opts)?;

                ATerm::Loop {
                    counter: counter.to_string(),
                    conditional,
                    block,
                }
            }
            Term::Break => {
                if return_opts.loop_returns {
                    ATerm::Break
                } else {
                    return Err(AParserError(
                        format!("Cannot break from outside loop."),
                        FileLocation::None,
                    ));
                }
            }
            Term::Continue => {
                if return_opts.loop_returns {
                    ATerm::Continue
                } else {
                    return Err(AParserError(
                        format!("Cannot continue from outside loop."),
                        FileLocation::None,
                    ));
                }
            }
            Term::Call { value } => {
                let value = aparse_operandexpression(value, &ds, gd)?;
                ATerm::Call { value }
            }
        };

        a_terms.push(a_term);
    }

    return Ok(ATermBlock::A { terms: a_terms });
}

pub fn aparse(program: &Program) -> Result<AProgram, AParserError> {
    let mut names = HashSet::new();
    let mut gd = GlobalData::new();
    let mut structs = Vec::new();
    let mut functions = Vec::new();

    for _struct in &program.structs {
        if names.contains(&_struct.name) {
            return Err(AParserError(
                format!("Global object {} has multiple definitions.", _struct.name),
                FileLocation::None,
            ));
        } else {
            names.insert(&_struct.name);
        }

        let name = _struct.name.clone();
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();

        for prop in &_struct.properties {
            let name = prop.identity.clone();
            let _type = AType::from_type_nyd(&prop.argtype, &mut gd);
            let field = AVarDef { name, _type };
            fields.insert(field.name.to_owned(), field);
        }

        for method in &_struct.methods {
            let returntype = AType::from_type_nyd(&method.returntype, &mut gd);
            let name = method.name.clone();

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
                FileLocation::None,
            ));
        } else {
            names.insert(&func.name);
        }

        let returntype = AType::from_type_nyd(&func.returntype, &mut gd);
        let name = func.name.clone();

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
        });

        gd.functions.insert(func.name.clone(), a_func.clone());
        functions.push(a_func);
    }

    for undefined in gd.not_yet_defined.clone() {
        let mut a_type_op = None;
        if let AType::NotYetDefined(ref _type) = *undefined.borrow() {
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

            a_type_op = Some(a_type)
        }

        if let Some(a_type) = a_type_op {
            undefined.swap(&a_type);
        }
    }

    for (_name, func) in &gd.functions {
        let mut new_a_termblock = None;
        if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
            if let ATermBlock::NotYetEvaluated(ref block) = *a_termblock.borrow() {
                let return_specs = if func.returntype.borrow().is_nulldef(&gd) {
                    ReturnOpts {
                        expected_type: None,
                        loop_returns: false,
                        require_explicit: false,
                    }
                } else {
                    ReturnOpts {
                        expected_type: Some(func.returntype.clone()),
                        loop_returns: false,
                        require_explicit: false,
                    }
                };
                new_a_termblock = Some(aparse_termblock(
                    block,
                    &DataScope::from_func_args(func),
                    &gd,
                    &return_specs,
                )?);
            }
        }

        if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
            if let Some(some) = new_a_termblock {
                a_termblock.replace(some);
            }
        }
    }

    for (_name, _struct) in &gd.structs {
        for (_name, func) in &_struct.methods {
            let mut new_a_termblock = None;
            if let AFuncBlock::TermsLang(ref a_termblock) = func.block {
                if let ATermBlock::NotYetEvaluated(ref block) = *a_termblock.borrow() {
                    let return_specs = if func.returntype.borrow().is_nulldef(&gd) {
                        ReturnOpts {
                            expected_type: None,
                            loop_returns: false,
                            require_explicit: false,
                        }
                    } else {
                        ReturnOpts {
                            expected_type: Some(func.returntype.clone()),
                            loop_returns: false,
                            require_explicit: false,
                        }
                    };
                    new_a_termblock = Some(aparse_termblock(
                        block,
                        &DataScope::from_func_args_this(&func, _struct.clone()),
                        &gd,
                        &return_specs,
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

    let a_program = AProgram { structs, functions };

    Ok(a_program)
}
