use crate::errors::{ActiveParserError, FileLocation};
use crate::parser::parse_operand_block::OperandExpression;
use crate::parser::{Object, ObjectType, Term, Type};
use std::collections::HashMap;

pub enum ActiveParse {
    File {
        active_terms: Vec<ActiveParse>,
    },
    Function {
        name: String,
        active_terms: Vec<ActiveParse>,
    },
    Struct {
        name: String,
        properties: Vec<Property>,
    },
    VarDeclation {
        name: String,
        value: ActiveOperandExpression,
    },
    VarUpdate {
        name: String,
        value: ActiveOperandExpression,
    },
    Return {
        value: ActiveOperandExpression,
    },
    If {
        conditional: ActiveOperandExpression,
        active_terms: Vec<ActiveParse>,
    },
    Loop {
        counter: String,
        conditional: String,
        active_terms: Vec<ActiveParse>,
    },
    ReadLn {
        object: ActiveObject,
    },
    Break,
    Continue,
    Call {
        operand_expression: ActiveOperandExpression,
    },
}

enum ActiveOperandExpression {
    Unary {
        operand: UnaryOperand,
        val: Box<OperandExpression>,
    },
    Binary {
        operand: BinaryOperand,
        left: Box<ActiveOperandExpression>,
        right: Box<ActiveOperandExpression>,
    },
    Literal(ActiveLiteral),
    Object(ActiveObject),
    Create(ActivateObjectCreate),
}

enum BinaryOperand {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,

    Equal,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
    And,
    Or,
}

enum UnaryOperand {
    Not,
}

enum ActiveLiteral {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

enum ActivateObjectCreate {}

enum ActiveObject {}

#[derive(Debug)]
struct VarRegistry<'a> {
    tree_connection: VarRegistryTreeConnection<'a>,
    objects: HashMap<u32, ObjectOption>,
}

impl<'a> VarRegistry<'a> {
    fn new(base_types: Vec<&str>) -> Self {
        let mut new = Self {
            tree_connection: VarRegistryTreeConnection::Base,
            objects: HashMap::new(),
        };

        for base_type in base_types {
            new.add_struct(base_type.to_string(), Vec::new());
        }

        return new;
    }

    fn create_child(&mut self) -> VarRegistry<'_> {
        VarRegistry {
            tree_connection: VarRegistryTreeConnection::Child { parent: self },
            objects: HashMap::new(),
        }
    }

    fn add_var(&mut self, name: String, _type: &Type) -> Result<u32, ActiveParserError> {
        let key = self.uid();
        let val = ObjectOption::Var {
            _type: convert_type_to_registry_type(_type, &self)?,
            name,
        };
        self.objects.insert(key, val);

        return Ok(key);
    }

    fn add_function(
        &mut self,
        name: String,
        args: Vec<VarRegistryVarType>,
        returntype: u32,
    ) -> Result<(), ActiveParserError> {
        let key = self.uid();
        let val = ObjectOption::Func {
            args,
            returntype,
            name,
        };
        self.objects.insert(key, val);

        return Ok(());
    }

    fn add_struct(&mut self, name: String, properties: Vec<Property>) {
        let uid = self.uid();
        self.objects
            .insert(uid, ObjectOption::Struct { name, properties });
    }

    fn get_id_of_type(&self, _type: &Type) -> Result<u32, ActiveParserError> {
        match _type {
            Type::Array(_type) => Err(ActiveParserError(
                "Must be type root, arrays not allowed.".to_string(),
                FileLocation::None,
            )),
            Type::Object { object } => self.get_id_of_object(object),
        }
    }

    fn get_id_of_object(&self, object: &Object) -> Result<u32, ActiveParserError> {
        match &object.kind {
            ObjectType::Identity(string) => self.get_id_of_string(string),
            _ => Err(ActiveParserError(
                "Expected object to be identity".to_string(),
                FileLocation::None,
            )),
        }
    }

    fn get_id_of_string(&self, string: &String) -> Result<u32, ActiveParserError> {
        for (id, object) in &self.objects {
            let name = match object {
                ObjectOption::Var { name, .. } => name,
                ObjectOption::Struct { name, .. } => name,
                ObjectOption::Func { name, .. } => name,
            };

            if name == string {
                return Ok(*id);
            }
        }

        match self.tree_connection {
            VarRegistryTreeConnection::Child { parent } => parent.get_id_of_string(string),
            VarRegistryTreeConnection::Base => {
                return Err(ActiveParserError(
                    format!("No object with name '{}' found", string),
                    FileLocation::None,
                ));
            }
        }
    }

    fn uid(&self) -> u32 {
        return rand::random();
    }
}

#[derive(Debug)]
enum ObjectOption {
    Var {
        _type: VarRegistryVarType,
        name: String,
    },
    Struct {
        properties: Vec<Property>,
        name: String,
    },
    Func {
        args: Vec<VarRegistryVarType>,
        returntype: u32,
        name: String,
    },
}

#[derive(Debug)]
enum VarRegistryTreeConnection<'a> {
    Child { parent: &'a VarRegistry<'a> },
    Base,
}

#[derive(Debug, Clone)]
struct Property {}

#[derive(Debug, Clone)]
enum VarRegistryVarType {
    Array(Box<VarRegistryVarType>),
    Base(u32),
}

fn convert_type_to_registry_type(
    _type: &Type,
    var_registry: &VarRegistry<'_>,
) -> Result<VarRegistryVarType, ActiveParserError> {
    match _type {
        Type::Array(_type) => Ok(VarRegistryVarType::Array(Box::new(
            convert_type_to_registry_type(_type, &var_registry)?,
        ))),
        Type::Object { .. } => Ok(VarRegistryVarType::Base(
            var_registry.get_id_of_type(_type)?,
        )),
    }
}

fn activate_print_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Print { ln, operand_block } = term {
        todo!("print term")
    } else {
        Err(ActiveParserError(
            "Expected print term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_var_declaration_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::DeclareVar {
        name,
        vartype,
        value,
    } = term
    {
        var_registry.add_var(name.clone(), &vartype)?;
        Ok(ActiveParse::VarDeclation {
            name,
            value: activate_operand_block(value)?,
        })
    } else {
        Err(ActiveParserError(
            "Expected variable declaration term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_operand_block(
    block: OperandExpression,
) -> Result<ActiveOperandExpression, ActiveParserError> {
    println!("WARNING: IGNORED OPERAND EXPRESSION");
    Ok(ActiveOperandExpression::Literal(ActiveLiteral::Int(1)))
}

fn activate_return_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Return { value } = term {
        todo!("return term")
    } else {
        Err(ActiveParserError(
            "Expected return term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_var_update_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::UpdateVar {
        var,
        set_operator,
        value,
    } = term
    {
        todo!("update term")
    } else {
        Err(ActiveParserError(
            "Expected update term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_if_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::If {
        conditional,
        block,
        else_block,
    } = term
    {
        todo!("if term")
    } else {
        Err(ActiveParserError(
            "Expected if term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_loop_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Loop {
        counter,
        conditional,
        block,
    } = term
    {
        todo!("loop term")
    } else {
        Err(ActiveParserError(
            "Expected loop term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_readln_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::ReadLn { var } = term {
        todo!("readln term")
    } else {
        Err(ActiveParserError(
            "Expected readln term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_break_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Break {} = term {
        todo!("break term")
    } else {
        Err(ActiveParserError(
            "Expected break term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_continue_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Continue {} = term {
        todo!("continue term")
    } else {
        Err(ActiveParserError(
            "Expected continue term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_call_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Call { value } = term {
        Ok(ActiveParse::Call {
            operand_expression: activate_operand_block(value)?,
        })
    } else {
        Err(ActiveParserError(
            "Expected call term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activeate_func_parse(
    func_term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Func {
        name,
        returntype,
        args,
        block,
    } = func_term
    {
        let func_return = var_registry.get_id_of_type(&returntype);

        let identity_args = {
            let mut identity_args = Vec::new();

            for arg in &args {
                identity_args.push(convert_type_to_registry_type(&arg.argtype, var_registry)?)
            }

            identity_args
        };

        var_registry.add_function(name.to_owned(), identity_args, func_return?)?;

        let mut var_registry = var_registry.create_child();

        for arg in &args {
            var_registry.add_var(arg.identity.to_owned(), &arg.argtype)?;
        }

        let mut active_terms = Vec::<ActiveParse>::new();
        if let Term::Block { terms } = *block {
            for term in terms {
                let active_term = match term {
                    Term::Block { .. } => {
                        return Err(ActiveParserError(
                            "Invalid floating block in function".to_string(),
                            FileLocation::None,
                        ))
                    }
                    Term::Func { .. } => activeate_func_parse(term.clone(), &mut var_registry),
                    Term::Print { .. } => activate_print_parse(term.clone(), &mut var_registry),
                    Term::DeclareVar { .. } => {
                        activate_var_declaration_parse(term.clone(), &mut var_registry)
                    }
                    Term::Return { .. } => activate_return_parse(term.clone(), &mut var_registry),
                    Term::UpdateVar { .. } => {
                        activate_var_update_parse(term.clone(), &mut var_registry)
                    }
                    Term::If { .. } => activate_if_parse(term.clone(), &mut var_registry),
                    Term::Loop { .. } => activate_loop_parse(term.clone(), &mut var_registry),
                    Term::ReadLn { .. } => activate_readln_parse(term.clone(), &mut var_registry),
                    Term::Break => activate_break_parse(term.clone(), &mut var_registry),
                    Term::Continue => activate_continue_parse(term.clone(), &mut var_registry),
                    Term::Call { .. } => activate_call_parse(term.clone(), &mut var_registry),
                    Term::Struct { .. } => {
                        return Err(ActiveParserError(
                            "Cannot declare class within a function".to_string(),
                            FileLocation::None,
                        ))
                    }
                };

                active_terms.push(active_term?);
            }
        } else {
            return Err(ActiveParserError(
                "Expected block as function block".to_string(),
                FileLocation::None,
            ));
        }

        return Ok(ActiveParse::Function { name, active_terms });
    } else {
        Err(ActiveParserError(
            "Expected function term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_struct_parse(
    struct_term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Struct { name, properties } = struct_term {
        let active_properties = {
            let mut active_properties = Vec::<Property>::new();
            for prop in properties {
                todo!("active properties")
            }

            active_properties
        };

        var_registry.add_struct(name.clone(), active_properties.clone());

        return Ok(ActiveParse::Struct {
            name,
            properties: active_properties,
        });
    } else {
        Err(ActiveParserError(
            "Expected struct term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_file_block(file_block: Term) -> Result<ActiveParse, ActiveParserError> {
    let mut var_registry = VarRegistry::new(vec!["null", "str"]);
    let mut var_registry = var_registry.create_child();

    let mut active_terms = Vec::<ActiveParse>::new();

    if let Term::Block { terms } = file_block {
        for term in terms.into_iter() {
            let out = match term {
                Term::Func { .. } => activeate_func_parse(term, &mut var_registry),
                Term::Struct { .. } => activate_struct_parse(term, &mut var_registry),
                _ => todo!(),
            };

            active_terms.push(out?);
        }

        Ok(ActiveParse::File { active_terms })
    } else {
        Err(ActiveParserError(
            "Expected block as wrapper parse object".to_string(),
            FileLocation::None,
        ))
    }
}

pub fn activate_parse(program: Term) -> Result<ActiveParse, ActiveParserError> {
    activate_file_block(program)
}
