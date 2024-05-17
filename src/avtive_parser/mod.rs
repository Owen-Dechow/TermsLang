use crate::errors::{ActiveParserError, FileLocation};
use crate::parser::{Term, Type, VarSigniture};

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
        methods: Vec<Method>,
    },
    VarDeclation,
    VarUpdate,
    Return,
    UpdateVar,
    If,
    Loop,
    ReadLn,
    Break,
    Continue,
    Call,
}

struct VarRegistry<'a> {
    tree_connection: VarRegistryTreeConnection<'a>,
    object: HashMap<u32, ObjectOption<'a>>,
}

impl<'a> VarRegistry<'a> {
    fn new() -> Self {
        Self {
            tree_connection: VarRegistryTreeConnection::Base { uid_counter: 0 },
            object: HashMap::new(),
        }
    }

    fn create_child(&mut self) -> VarRegistry<'_> {
        VarRegistry {
            tree_connection: VarRegistryTreeConnection::Child { parent: self },
            object: HashMap::new(),
        }
    }

    fn add_var(&mut self, name: String, _type: &Type) {
        todo!()
    }

    fn collect_args(&self, args: &Vec<VarSigniture>) -> Vec<VarRegistryVarType> {
        todo!()
    }

    fn add_function_to_parent(
        &mut self,
        name: String,
        args: Vec<VarRegistryVarType>,
        returntype: u32,
    ) {
        todo!()
    }

    fn add_struct_to_parent(
        &mut self,
        name: String,
        properties: Vec<Property>,
        methods: Vec<Method>,
    ) {
        todo!()
    }

    fn get_id_of_type(&self, _type: &Type) -> Result<u32, ActiveParserError> {
        todo!("get id")
    }
}

enum ObjectOption<'a> {
    Var {
        var_type: VarRegistryVarType,
        name: String,
    },
    TypeVar {
        name: String,
    },
    Struct {
        parent: Option<&'a ObjectOption<'a>>,
    },
    Func {
        args: Vec<VarRegistryVarType>,
        returntype: u32,
        name: String,
    },
}

enum VarRegistryTreeConnection<'a> {
    Child { parent: &'a VarRegistry<'a> },
    Base { uid_counter: u32 },
}

struct Method {
    is_static: bool,
    func: ActiveParse,
}

struct Property {}

enum VarRegistryVarType {
    List(Box<VarRegistryVarType>),
    Base(u32),
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
        todo!("declare var term")
    } else {
        Err(ActiveParserError(
            "Expected variable declaration term".to_string(),
            FileLocation::None,
        ))
    }
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
        todo!("call term")
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
        let mut var_registry = var_registry.create_child();

        for arg in &args {
            var_registry.add_var(arg.identity.to_owned(), &arg.argtype);
        }

        let func_return = var_registry.get_id_of_type(&returntype)?;

        let args = var_registry.collect_args(&args);

        var_registry.add_function_to_parent(name.to_owned(), args, func_return);

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
    if let Term::Struct {
        name,
        properties,
        methods,
    } = struct_term
    {
        let mut var_registry = var_registry.create_child();
        let active_methods = {
            let mut active_methods = Vec::<Method>::new();
            for method in methods {
                let active_func = activeate_func_parse(method.func, &mut var_registry)?;
                active_methods.push(Method {
                    is_static: method.is_static,
                    func: active_func,
                })
            }

            active_methods
        };
        let active_properties = {
            let mut active_properties = Vec::<Property>::new();
            for prop in properties {
                todo!("active properties")
            }

            active_properties
        };

        var_registry.add_struct_to_parent(name, active_properties, active_methods);

        todo!()
    } else {
        Err(ActiveParserError(
            "Expected struct term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_file_block(file_block: Term) -> Result<ActiveParse, ActiveParserError> {
    let mut var_registry = VarRegistry::new();
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
