use crate::{
    errors::{AParserError, FileLocation},
    lexer::tokens::{Operator, Token, TokenType},
    parser::{
        parse_operand_block::OperandExpression, Object, ObjectType, Program, Term, TermBlock, Type,
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

            int_type: AStruct::tmp_none().into(),
            bool_type: AStruct::tmp_none().into(),
            string_type: AStruct::tmp_none().into(),
            float_type: AStruct::tmp_none().into(),
            null_type: AStruct::tmp_none().into(),
        };

        new.int_type = new.add_root_struct(
            "int",
            &[
                ("@str", &[]),
                ("@int", &[]),
                ("@float", &[]),
                ("@bool", &[]),
                ("@new", &["int"]),
                ("@add", &["int"]),
                ("@sub", &["int"]),
                ("@mult", &["int"]),
                ("@div", &["int"]),
                ("@mod", &["int"]),
                ("@exp", &["int"]),
                ("@eq", &["int"]),
                ("@gt", &["int"]),
                ("@gteq", &["int"]),
                ("@lt", &["int"]),
                ("@lteq", &["int"]),
            ],
        );
        new.null_type = new.add_root_struct(
            "null",
            &[
                ("@str", &[]),
                ("@int", &[]),
                ("@float", &[]),
                ("@new", &[]),
                ("@bool", &[]),
                ("@eq", &["int"]),
            ],
        );
        new.float_type = new.add_root_struct(
            "float",
            &[
                ("@str", &[]),
                ("@int", &[]),
                ("@float", &[]),
                ("@bool", &[]),
                ("@new", &["float"]),
                ("@add", &["float"]),
                ("@sub", &["float"]),
                ("@mult", &["float"]),
                ("@div", &["float"]),
                ("@mod", &["float"]),
                ("@exp", &["float"]),
                ("@eq", &["float"]),
                ("@gt", &["float"]),
                ("@gteq", &["float"]),
                ("@lt", &["float"]),
                ("@lteq", &["float"]),
            ],
        );
        new.bool_type = new.add_root_struct(
            "bool",
            &[
                ("@str", &[]),
                ("@int", &[]),
                ("@float", &[]),
                ("@bool", &[]),
                ("@new", &["bool"]),
                ("@not", &["bool"]),
                ("@and", &["bool"]),
                ("@or", &["bool"]),
            ],
        );
        new.string_type = new.add_root_struct(
            "str",
            &[
                ("@str", &[]),
                ("@int", &[]),
                ("@float", &[]),
                ("@bool", &[]),
                ("@new", &["str"]),
                ("@len", &[]),
                ("@add", &["str"]),
                ("@mult", &["int"]),
                ("@mod", &["str"]),
            ],
        );

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

    fn add_root_struct(&mut self, name: &str, funcs: &[(&str, &[&str])]) -> Rc<AStruct> {
        let mut a_funcs = HashMap::new();
        for func in funcs {
            let mut args = Vec::new();
            for arg in func.1 {
                let a_arg = AVarDef {
                    name: String::new(),
                    _type: self.create_forward_ref(arg),
                };

                args.push(a_arg);
            }

            let a_func = AFunc {
                name: func.0.to_string(),
                returntype: self.create_forward_ref(name),
                block: AFuncBlock::Rust,
                args,
            };

            a_funcs.insert(func.0.to_string(), a_func.into());
        }

        let a_struct = Rc::new(AStruct::System {
            name: name.to_string(),
            fields: HashMap::new(),
            methods: a_funcs,
        });

        self.structs.insert(name.to_string(), a_struct.clone());
        return a_struct;
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

struct DataScope {
    parent: Option<Rc<DataScope>>,
    vars: HashMap<String, Rc<RefCell<AType>>>,
}
impl DataScope {
    fn new() -> Self {
        DataScope {
            parent: None,
            vars: HashMap::new(),
        }
    }

    fn from_parent(parent: &Rc<Self>) -> Self {
        DataScope {
            parent: Some(parent.clone()),
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
}

#[derive(Debug)]
pub struct AProgram {
    structs: Vec<Rc<AStruct>>,
    functions: Vec<Rc<AFunc>>,
}

#[derive(Debug)]
enum ATermBlock {
    A { terms: Vec<ATerm> },
    NotYetEvaluated(TermBlock),
}

#[derive(Debug)]
enum ATerm {
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
}

enum AType {
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
            _ => panic!(),
        }
    }
}

impl Debug for AType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArrayObject(arg0) => f.debug_tuple(&format!("Type({:?}[])", arg0)).finish(),
            Self::StructObject(arg0) => f
                .debug_tuple(&format!(
                    "Type($() {})",
                    match arg0.as_ref() {
                        AStruct::System { name, .. } => name,
                        AStruct::User { name, .. } => name,
                    }
                ))
                .finish(),
            Self::StructDefRef(arg0) => f
                .debug_tuple(&format!(
                    "Type({})",
                    match arg0.as_ref() {
                        AStruct::System { name, .. } => name,
                        AStruct::User { name, .. } => name,
                    }
                ))
                .finish(),
            Self::FuncDefRef(arg0) => f.debug_tuple(&format!("Type(func {})", arg0.name)).finish(),
            Self::NotYetDefined(..) => f.debug_tuple("Type(NotYetDefined)").finish(),
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
            AType::ArrayObject(rc) => AObject::from_object_sub_array(object, parent_type, ds, gd),
            AType::StructObject(rc) => AObject::from_object_sub_struct(object, parent_type, ds, gd),
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
                    call.args.len(),
                    func.args.len()
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
                return Err(AParserError(format!("Type Missmatch"), FileLocation::None));
            }

            acall.args.push(a_arg);
        }

        let sub = if let Some(sub) = &object.sub {
            todo!();
        } else {
            None
        };

        Ok(AObject {
            kind: AObjectType::Call(acall),
            sub,
            _type: func.returntype.clone(),
            loc: object.loc.clone(),
        })
    }

    fn from_object_sub_array(
        object: &Object,
        parent_type: &AType,
        ds: &DataScope,
        gd: &GlobalData,
    ) -> Result<AObject, AParserError> {
        todo!()
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
                let (name, fields, methods) = match **astruct {
                    AStruct::System {
                        ref name,
                        ref fields,
                        ref methods,
                    } => (name, fields, methods),
                    AStruct::User {
                        ref name,
                        ref fields,
                        ref methods,
                    } => (name, fields, methods),
                };

                let (_type, connected_instance_type) = match fields.get(id) {
                    Some(some) => (some._type.clone(), None),
                    None => match methods.get(id) {
                        Some(some) => (
                            RefCell::new(AType::FuncDefRef(some.clone())).into(),
                            Some(parent_type),
                        ),
                        None => {
                            return Err(AParserError(
                                format!("Struct object {} has no field or method {}.", name, id),
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
            ObjectType::Call(call) => todo!(),
            ObjectType::Index(operand_expression) => todo!(),
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
struct AVarDef {
    name: String,
    _type: Rc<RefCell<AType>>,
}

#[derive(Debug)]
enum AFuncBlock {
    Rust,
    TermsLang(Rc<RefCell<ATermBlock>>),
}

#[derive(Debug)]
struct AFunc {
    name: String,
    returntype: Rc<RefCell<AType>>,
    block: AFuncBlock,
    args: Vec<AVarDef>,
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
struct AOperandExpression {
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
}

#[derive(Debug)]
enum AStruct {
    System {
        name: String,
        fields: HashMap<String, AVarDef>,
        methods: HashMap<String, Rc<AFunc>>,
    },
    User {
        name: String,
        fields: HashMap<String, AVarDef>,
        methods: HashMap<String, Rc<AFunc>>,
    },
}
impl AStruct {
    fn tmp_none() -> Self {
        Self::System {
            name: String::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}

struct ReturnSpecs {
    expected_type: Option<Rc<RefCell<AType>>>,
    loop_returns: bool,
    require_explicit: bool,
}

fn aparse_operandexpression(
    operand_expression: &OperandExpression,
    ds: &DataScope,
    gd: &GlobalData,
) -> Result<AOperandExpression, AParserError> {
    match operand_expression {
        OperandExpression::Unary { operand, val } => todo!(),
        OperandExpression::Binary {
            operand,
            left,
            right,
        } => todo!(),
        OperandExpression::Dot { left, right } => todo!(),
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
                _type: a_object._type.clone(),
                value: AOperandExpressionValue::Object(a_object),
            });
        }
        OperandExpression::Create(_) => todo!(),
    }
}

fn aparse_termblock(
    block: &TermBlock,
    parent_ds: Rc<DataScope>,
    gd: &GlobalData,
    return_opts: &ReturnSpecs,
) -> Result<ATermBlock, AParserError> {
    let mut a_terms = Vec::new();
    let mut ds = DataScope::from_parent(&parent_ds);

    for term in &block.terms {
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
                        format!("Missmatched types"),
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
                            format!("Missmatched types"),
                            FileLocation::None,
                        ));
                    }
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
                    return Err(AParserError(format!("Missmatched types"), var.loc.clone()));
                }

                ATerm::UpdateVar { value, var }
            }
            Term::If {
                conditional,
                block,
                else_block,
            } => {
                let conditional = aparse_operandexpression(conditional, &ds, gd)?;

                todo!()
            }
            Term::Loop {
                counter,
                conditional,
                block,
            } => todo!(),
            Term::Break => todo!(),
            Term::Continue => todo!(),
            Term::Call { value } => todo!(),
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
            todo!();
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

        let a_struct = Rc::new(AStruct::User {
            name,
            fields,
            methods,
        });

        gd.structs.insert(_struct.name.clone(), a_struct.clone());
        structs.push(a_struct);
    }

    for func in &program.functions {
        if names.contains(&func.name) {
            todo!();
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
                    ReturnSpecs {
                        expected_type: None,
                        loop_returns: false,
                        require_explicit: false,
                    }
                } else {
                    ReturnSpecs {
                        expected_type: Some(func.returntype.clone()),
                        loop_returns: false,
                        require_explicit: false,
                    }
                };
                new_a_termblock = Some(aparse_termblock(
                    block,
                    DataScope::new().into(),
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

    let a_program = AProgram { structs, functions };

    Ok(a_program)
}
