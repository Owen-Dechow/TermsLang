use rand::random;
use std::{collections::HashMap, env, rc::Rc};

use crate::{
    errors::{FileLocation, RuntimeError},
    lexer::tokens::{Token, TokenType},
    parser::{Function, Object, ObjectCreate, ObjectType, Struct, Type},
};

use super::{
    interpret_operand_expression, syntax, var_registry::VariableRegistry, ArrayObject, DataCase,
    DataObject, ExitMethod, FuncBlock, FuncDef, RootFunc, RootObject, RootType, StructDef,
    StructObject, TypeResolve,
};

#[derive(Debug)]
pub struct GarbageCollector {
    pub objects: HashMap<u32, DataCase>,
    pub global_structs: HashMap<u32, Rc<StructDef>>,
    pub global_methods: HashMap<u32, Rc<FuncDef>>,
    pub root_type_map: HashMap<RootType, u32>,
    pub command_line_args: u32,
}
impl GarbageCollector {
    pub fn new(root_vr: &mut VariableRegistry) -> GarbageCollector {
        let mut gc = GarbageCollector {
            objects: HashMap::new(),
            global_structs: HashMap::new(),
            global_methods: HashMap::new(),
            root_type_map: HashMap::new(),
            command_line_args: random(),
        };

        gc.add_root_type(
            "int",
            RootType::Int,
            vec![syntax::TO_STRING_FUNC, syntax::TO_INT_FUNC],
            root_vr,
        );
        gc.add_root_type(
            "bool",
            RootType::Bool,
            vec![syntax::TO_STRING_FUNC, syntax::TO_INT_FUNC],
            root_vr,
        );
        gc.add_root_type(
            "float",
            RootType::Float,
            vec![syntax::TO_STRING_FUNC, syntax::TO_INT_FUNC],
            root_vr,
        );
        gc.add_root_type(
            "str",
            RootType::String,
            vec![syntax::TO_STRING_FUNC, syntax::TO_INT_FUNC],
            root_vr,
        );
        gc.add_root_type(
            "null",
            RootType::Null,
            vec![syntax::TO_STRING_FUNC, syntax::TO_INT_FUNC],
            root_vr,
        );

        gc.add_root_func("@readln", RootFunc::ReadLn, Vec::new(), root_vr);

        let mut command_line_args = Vec::new();
        for arg in env::args() {
            let key = random();
            command_line_args.push(key);

            gc.objects.insert(
                key,
                DataCase {
                    ref_count: 1,
                    data: DataObject::RootObject(RootObject::String(arg)),
                },
            );
        }

        gc.objects.insert(
            gc.command_line_args,
            DataCase {
                ref_count: 0,
                data: DataObject::ArrayObject(ArrayObject(command_line_args)),
            },
        );

        return gc;
    }

    fn add_root_type(
        &mut self,
        name: &str,
        root_type: RootType,
        methods: Vec<&str>,
        root_vr: &mut VariableRegistry,
    ) {
        let key = random();
        self.global_structs.insert(
            key,
            StructDef::Root {
                _type: root_type.clone(),
                name: name.to_string(),
                methods: methods.into_iter().map(|x| x.to_string()).collect(),
            }
            .into(),
        );
        root_vr.vars.insert(name.to_string(), key);
        self.root_type_map.insert(root_type, key);
    }

    fn add_root_func(
        &mut self,
        name: &str,
        func: RootFunc,
        args: Vec<(String, TypeResolve)>,
        root_vr: &mut VariableRegistry,
    ) {
        let func_def = FuncDef {
            name: name.to_string(),
            args,
            block: FuncBlock::Root(func),
        };

        let key = random();
        self.global_methods.insert(key, func_def.into());
        root_vr.vars.insert(name.to_string(), key);
    }

    pub fn add_struct(
        &mut self,
        _struct: Struct,
        root_vr: &mut VariableRegistry,
    ) -> Result<(), RuntimeError> {
        let mut properties = HashMap::new();
        for property in _struct.properties {
            properties.insert(
                property.identity,
                self.resolve_type(&property.argtype, root_vr)?,
            );
        }

        let mut methods = HashMap::new();
        for method in _struct.methods {
            let args = {
                let mut args = Vec::new();
                for arg in method.args {
                    args.push((arg.identity, self.resolve_type(&arg.argtype, root_vr)?));
                }
                args
            };
            let block = FuncBlock::User(method.block);

            let func_def = FuncDef {
                name: method.name.clone(),
                args,
                block,
            };

            let key = random();
            self.global_methods.insert(key, func_def.into());

            methods.insert(method.name, key);
        }

        let struct_def = StructDef::User {
            properties,
            methods,
            name: _struct.name.clone(),
        };

        let key = random();
        self.global_structs.insert(key, struct_def.into());
        root_vr.vars.insert(_struct.name, key);

        return Ok(());
    }

    pub fn add_function(
        &mut self,
        func: Function,
        root_vr: &mut VariableRegistry,
    ) -> Result<(), RuntimeError> {
        let args = {
            let mut args = Vec::new();
            for arg in func.args {
                args.push((arg.identity, self.resolve_type(&arg.argtype, root_vr)?));
            }
            args
        };
        let block = FuncBlock::User(func.block);

        let func_def = FuncDef {
            name: func.name.clone(),
            args,
            block,
        };

        let key = random();
        self.global_methods.insert(key, func_def.into());
        root_vr.vars.insert(func.name, key);
        return Ok(());
    }

    fn resolve_type(
        &self,
        _type: &Type,
        root_vr: &VariableRegistry,
    ) -> Result<TypeResolve, RuntimeError> {
        match _type {
            Type::Array(_type) => Ok(TypeResolve::Array(Box::new(
                self.resolve_type(&*_type, root_vr)?,
            ))),
            Type::Object { object } => match &object.kind {
                ObjectType::Identity(id) => match object.sub {
                    Some(_) => Err(RuntimeError(
                        "Expected type".to_string(),
                        FileLocation::None,
                    )),
                    None => Ok(TypeResolve::Standard(root_vr.resolve_string(id)?)),
                },
                _ => Err(RuntimeError(
                    "Expected type".to_string(),
                    FileLocation::None,
                )),
            },
        }
    }

    pub fn resolve_object(
        &mut self,
        object: &Object,
        vr: &VariableRegistry,
        root_vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        match &object.kind {
            ObjectType::Identity(id) => {
                let key = vr.resolve_string(id)?;

                match &object.sub {
                    Some(sub) => self.resolve_sub_object(key, None, &*sub, vr, root_vr),
                    None => Ok(key),
                }
            }
            _ => todo!(),
        }
    }

    fn resolve_root_sub(
        &mut self,
        root_def: Rc<StructDef>,
        object: &Object,
        id: &String,
        parent: u32,
        vr: &VariableRegistry,
        root_vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        match *root_def {
            StructDef::User { .. } => todo!(),
            StructDef::Root {
                ref _type,
                ref methods,
                ..
            } => {
                if methods.contains(id) {
                    match &object.sub {
                        Some(sub) => match &sub.kind {
                            ObjectType::Call(call) => {
                                let mut data_object = self.objects[&parent].data.clone();
                                let args = {
                                    let mut args = Vec::new();
                                    for arg in &call.args {
                                        args.push(interpret_operand_expression(
                                            &arg, self, vr, root_vr,
                                        )?)
                                    }
                                    args
                                };
                                let result =
                                    match data_object.call_method(self, id, args, root_vr)? {
                                        ExitMethod::ImplicitNullReturn => todo!(),
                                        ExitMethod::ExplicitReturn(id) => Ok(id),
                                        ExitMethod::LoopContinue => todo!(),
                                        ExitMethod::LoopBreak => todo!(),
                                    }?;

                                match &sub.sub {
                                    Some(sub) => self.resolve_sub_object(
                                        result,
                                        Some(parent),
                                        &*sub,
                                        vr,
                                        root_vr,
                                    ),
                                    None => Ok(result),
                                }
                            }
                            _ => Err(RuntimeError(
                                format!(
                                    "{}{}",
                                    "Root type functions must be called; alias creation,",
                                    " indexing, and object peeking is not allowed."
                                ),
                                FileLocation::None,
                            )),
                        },
                        None => Err(RuntimeError(
                            format!(
                                "{}{}",
                                "Root type functions must be called; alias creation,",
                                " indexing, and object peeking is not allowed."
                            ),
                            FileLocation::None,
                        )),
                    }
                } else {
                    Err(RuntimeError(
                        format!("{} no field or method found on struct", id),
                        FileLocation::None,
                    ))
                }
            }
        }
    }

    fn reslove_array_sub(
        &mut self,
        object: &Object,
        id: &String,
        parent: u32,
        vr: &VariableRegistry,
        root_vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        if syntax::ARRAY_METHODS.contains(&id.as_str()) {
            match &object.sub {
                Some(sub) => match &sub.kind {
                    ObjectType::Call(call) => {
                        let args = {
                            let mut args = Vec::new();
                            for arg in &call.args {
                                args.push(interpret_operand_expression(&arg, self, vr, root_vr)?)
                            }
                            args
                        };

                        let result = match DataObject::call_array_func(parent, self, id, args)? {
                            ExitMethod::ImplicitNullReturn => todo!(),
                            ExitMethod::ExplicitReturn(id) => Ok(id),
                            ExitMethod::LoopContinue => todo!(),
                            ExitMethod::LoopBreak => todo!(),
                        }?;

                        match &sub.sub {
                            Some(sub) => {
                                self.resolve_sub_object(result, Some(parent), &*sub, vr, root_vr)
                            }
                            None => Ok(result),
                        }
                    }
                    _ => Err(RuntimeError(
                        format!(
                            "{}{}",
                            "Array type functions must be called; alias creation, indexing,",
                            " and object peeking is not allowed."
                        ),
                        FileLocation::None,
                    )),
                },
                None => Err(RuntimeError(
                    format!(
                        "{}{}",
                        "Array type functions must be called; alias creation, indexing,",
                        " and object peeking is not allowed."
                    ),
                    FileLocation::None,
                )),
            }
        } else {
            Err(RuntimeError(
                format!("{} no field or method found on array", id),
                FileLocation::None,
            ))
        }
    }

    pub fn create_null_object(&mut self) -> u32 {
        let key = random();

        self.objects.insert(
            key,
            DataCase {
                ref_count: 0,
                data: DataObject::RootObject(RootObject::Null),
            },
        );

        return key;
    }

    pub fn resolve_sub_object(
        &mut self,
        parent: u32,
        parent_parent: Option<u32>,
        object: &Object,
        vr: &VariableRegistry,
        root_vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        match &object.kind {
            ObjectType::Identity(id) => match &self.objects[&parent].data {
                DataObject::StructObject(struct_object) => match struct_object.fields.get(id) {
                    Some(key) => match &object.sub {
                        Some(sub) => {
                            self.resolve_sub_object(*key, Some(parent), &*sub, vr, root_vr)
                        }
                        None => Ok(*key),
                    },
                    None => {
                        let struct_def = &self.global_structs[&struct_object._type];
                        if let StructDef::User { ref methods, .. } = **struct_def {
                            match methods.get(id) {
                                Some(func_def) => match &object.sub {
                                    Some(sub) => self.resolve_sub_object(
                                        *func_def,
                                        Some(parent),
                                        &*sub,
                                        vr,
                                        root_vr,
                                    ),
                                    None => Err(RuntimeError(
                                        format!(
                                            "Cannot reference function, {}, without calling",
                                            id
                                        ),
                                        FileLocation::None,
                                    )),
                                },
                                None => {
                                    return Err(RuntimeError(
                                        format!("No field or method {} on struct", id),
                                        FileLocation::None,
                                    ))
                                }
                            }
                        } else {
                            todo!()
                        }
                    }
                },
                DataObject::RootObject(root_object) => {
                    let root_def = root_object.get_root_type_def(self);
                    self.resolve_root_sub(root_def, object, id, parent, vr, root_vr)
                }
                DataObject::ArrayObject(..) => {
                    self.reslove_array_sub(object, id, parent, vr, root_vr)
                }
            },
            ObjectType::Call(call) => match self.global_methods.get(&parent) {
                Some(func_def) => {
                    let func_def = func_def.clone();
                    let args = {
                        let mut args = Vec::new();
                        for arg in &call.args {
                            args.push(interpret_operand_expression(&arg, self, vr, root_vr)?)
                        }
                        args
                    };

                    let result = if let Some(struct_id) = parent_parent {
                        func_def.struct_call(args, self, struct_id, root_vr)?
                    } else {
                        func_def.global_call(args, self, root_vr)?
                    };

                    let result = match result {
                        ExitMethod::ImplicitNullReturn => Ok(self.create_null_object()),
                        ExitMethod::ExplicitReturn(id) => Ok(id),
                        ExitMethod::LoopContinue => todo!(),
                        ExitMethod::LoopBreak => todo!(),
                    }?;

                    match &object.sub {
                        Some(sub) => {
                            self.resolve_sub_object(result, Some(parent), &*sub, vr, root_vr)
                        }
                        None => Ok(result),
                    }
                }
                None => Err(RuntimeError(
                    "Object is not a function".to_string(),
                    FileLocation::None,
                )),
            },
            ObjectType::Index(idx) => {
                let args = vec![interpret_operand_expression(idx, self, vr, root_vr)?];

                let parent_obj = &mut self.objects.get_mut(&parent).unwrap().data;
                let result = match parent_obj {
                    DataObject::StructObject(..) => {
                        todo!()
                    }
                    DataObject::RootObject(..) => {
                        return Err(RuntimeError(
                            "Cannot index into root object".to_string(),
                            FileLocation::None,
                        ))
                    }
                    DataObject::ArrayObject(..) => match DataObject::call_array_func(
                        parent,
                        self,
                        &syntax::INDEX_FUNC.to_string(),
                        args,
                    )? {
                        ExitMethod::ImplicitNullReturn => todo!(),
                        ExitMethod::ExplicitReturn(id) => Ok(id),
                        ExitMethod::LoopContinue => todo!(),
                        ExitMethod::LoopBreak => todo!(),
                    }?,
                };

                match &object.sub {
                    Some(sub) => self.resolve_sub_object(result, Some(parent), &*sub, vr, root_vr),
                    None => Ok(result),
                }
            }
        }
    }

    pub fn add_literal(&mut self, literal: &Token) -> Result<u32, RuntimeError> {
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
        self.objects.insert(
            key,
            DataCase {
                ref_count: 0,
                data: DataObject::RootObject(root),
            },
        );
        return Ok(key);
    }

    pub fn get_struct_def_from_id(&self, key: u32) -> Result<&StructDef, RuntimeError> {
        match self.global_structs.get(&key) {
            Some(struct_def) => Ok(struct_def),
            None => todo!(),
        }
    }

    pub fn create_object(
        &mut self,
        create: &ObjectCreate,
        vr: &VariableRegistry,
        root_vr: &VariableRegistry,
    ) -> Result<u32, RuntimeError> {
        let key = random();
        let _type = self.resolve_type(&create.kind, root_vr)?;
        let mut new_func = None;

        let data = match _type {
            TypeResolve::Array(_arr) => {
                if create.args.args.len() > 0 {
                    return Err(RuntimeError(
                        "Cannot have args in array creation.".to_string(),
                        FileLocation::None,
                    ));
                }

                DataObject::ArrayObject(ArrayObject(Vec::new()))
            }
            TypeResolve::Standard(std) => {
                let struct_def = self.get_struct_def_from_id(std)?.clone();
                match struct_def {
                    StructDef::Root { .. } => {
                        return Err(RuntimeError(
                            "Cannot directly construct root type".to_string(),
                            FileLocation::None,
                        ));
                    }
                    StructDef::User {
                        properties,
                        methods,
                        name: _name,
                    } => {
                        let mut fields = HashMap::new();

                        for (property, _type) in properties {
                            let null = self.create_null_object();
                            self.objects.get_mut(&null).unwrap().ref_count += 1;
                            fields.insert(property.clone(), null);
                        }

                        new_func = match methods.get(syntax::NEW_FUNC) {
                            Some(s) => Some(*s),
                            None => None,
                        };

                        DataObject::StructObject(StructObject {
                            fields,
                            _type: std,
                            self_key: key,
                        })
                    }
                }
            }
        };

        self.objects.insert(key, DataCase { ref_count: 0, data });

        if let Some(func_id) = new_func {
            let func = self.global_methods[&func_id].clone();

            let args = {
                let mut args = Vec::new();

                for arg in &create.args.args {
                    let arg = interpret_operand_expression(&arg, self, vr, root_vr)?;
                    args.push(arg);
                }

                args
            };

            func.struct_call(args, self, key, root_vr)?;
        }

        return Ok(key);
    }

    pub fn release(&mut self, key: u32) {
        if let Some(DataCase { ref_count: 0, .. }) = self.objects.get(&key) {
            self.objects.remove(&key);
        }
    }
}
