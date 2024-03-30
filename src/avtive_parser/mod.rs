use crate::errors::{ActiveParserError, FileLocation};
use crate::parser::{Term, Type, VarSigniture};

use std::collections::HashMap;

pub enum ActiveParse {
    File { active_terms: Vec<ActiveParse> },
    Function {},
    Class {},
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

    fn add_typevar(&mut self, name: String) {
        todo!()
    }

    fn collect_type_args<'b>(&self) -> Vec<ObjectOption<'b>> {
        todo!()
    }

    fn collect_args(&self, args: &Vec<VarSigniture>) -> Vec<VarRegistryVarType> {
        todo!()
    }

    fn add_function_to_parent(
        &mut self,
        name: String,
        type_args: Vec<ObjectOption>,
        args: Vec<VarRegistryVarType>,
        returntype: FuncReturnTypeOption,
    ) {
        todo!()
    }

    fn activate_func_return_option(
        &self,
        returntype: &Type,
    ) -> Result<FuncReturnTypeOption, ActiveParserError> {
        todo!()
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
    Class {
        parent: Option<&'a ObjectOption<'a>>,
    },
    Func {
        type_args: Vec<ObjectOption<'a>>,
        args: Vec<VarRegistryVarType>,
        returntype: FuncReturnTypeOption,
        name: String,
    },
}

enum VarRegistryTreeConnection<'a> {
    Child { parent: &'a VarRegistry<'a> },
    Base { uid_counter: u32 },
}

enum FuncReturnTypeOption {
    TypeArg(u32),
    Static(u32),
}

enum VarRegistryVarType {
    List(Box<VarRegistryVarType>),
    Base(u32),
}

fn activate_print_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_var_declaration_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_return_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_var_update_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_if_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_loop_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_readln_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_break_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_continue_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activate_call_parse(
    term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    todo!()
}

fn activeate_func_parse(
    func_term: Term,
    var_registry: &mut VarRegistry<'_>,
) -> Result<ActiveParse, ActiveParserError> {
    if let Term::Func {
        name,
        returntype,
        typeargs,
        args,
        block,
    } = func_term
    {
        let mut var_registry = var_registry.create_child();

        for arg in &args {
            var_registry.add_var(arg.identity.to_owned(), &arg.argtype);
        }

        for typearg in typeargs {
            var_registry.add_typevar(typearg);
        }

        let func_return = var_registry.activate_func_return_option(&returntype)?;

        let args = var_registry.collect_args(&args);
        let type_args = var_registry.collect_type_args();

        var_registry.add_function_to_parent(name.to_owned(), type_args, args, func_return);

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
                    Term::Class { .. } => {
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

        return Ok(ActiveParse::Function {});
    } else {
        Err(ActiveParserError(
            "Expected function term".to_string(),
            FileLocation::None,
        ))
    }
}

fn activate_file_block(file_block: Term) -> Result<ActiveParse, ActiveParserError> {
    let mut var_registry = VarRegistry::new();
    let active_terms = Vec::<ActiveParse>::new();

    if let Term::Block { terms } = file_block {
        for term in terms.into_iter() {
            let out = match term {
                Term::Func { .. } => activeate_func_parse(term, &mut var_registry),
                Term::Class { .. } => todo!(),
                _ => todo!(),
            };
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
