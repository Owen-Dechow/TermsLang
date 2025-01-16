use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::active_parser::{
    names as nms, ACall, AFunc, AFuncBlock, ALiteral, AObject, AObjectType, AOperandExpression,
    AOperandExpressionValue, AProgram, AStruct, ATerm, ATermBlock, AType,
};

struct ProgramBuilder {
    tape: Vec<CMD>,
    function_idxs: HashMap<u32, usize>,
    main_function: usize,
    non_indexed_refers: Vec<(usize, u32)>,
    non_indexed_loops: Vec<Vec<usize>>,
}
impl ProgramBuilder {
    fn new() -> Self {
        ProgramBuilder {
            tape: Vec::new(),
            function_idxs: HashMap::new(),
            main_function: 0,
            non_indexed_refers: Vec::new(),
            non_indexed_loops: Vec::new(),
        }
    }

    fn push(&mut self, cmd: CMD) -> usize {
        self.tape.push(cmd);
        return self.len() - 1;
    }

    fn len(&self) -> usize {
        self.tape.len()
    }

    fn split_scope(&mut self, defer_count: &mut u32, release_count: &mut Vec<u32>) {
        self.push(CMD::SplitScope);
        *defer_count += 1;

        if let Some(r) = release_count.last_mut() {
            *r += 1;
        }
    }

    fn release_scope(&mut self, defer_count: &mut u32, release_count: &mut Vec<u32>) {
        self.push(CMD::Release);
        *defer_count -= 1;

        if let Some(r) = release_count.last_mut() {
            *r -= 1;
        }
    }
}

#[derive(Debug)]
pub enum VarAdress {
    Index(String),
    Var(String),
}

#[derive(Debug)]
pub enum Literal {
    Int(i32),
    String(String),
    Float(f32),
    Bool(bool),
    Null,
}
impl Literal {
    fn from_aliteral(lit: &ALiteral) -> Self {
        match lit {
            ALiteral::Int(i) => Literal::Int(*i),
            ALiteral::Float(i) => Literal::Float(*i),
            ALiteral::String(i) => Literal::String(i.clone()),
            ALiteral::Bool(i) => Literal::Bool(*i),
        }
    }
}

#[derive(Debug)]
pub enum CMD {
    SplitScope,
    Release,
    RelaseN(u32),
    Defer(u32),
    Jump(usize),
    Push(VarAdress),
    Print,
    PrintLn,
    Let(String),
    Update,
    XIf,
    Refer(usize),
    InternalOp(String),
    PushLit(Literal),
    PushObj(Vec<String>),
    Burn,
}

pub struct FlatProgram {
    pub tape: Vec<CMD>,
    pub start_point: usize,
}
impl FlatProgram {
    pub fn prettify(&self) -> String {
        let mut string = format!("{}\n", self.start_point);

        for (idx, x) in (&self.tape).into_iter().enumerate() {
            string += &format!("{idx:5}: {x:?}\n");
        }

        return string;
    }
}

fn add_block(
    pb: &mut ProgramBuilder,
    block: &ATermBlock,
    defer_count: &mut u32,
    release_count: &mut Vec<u32>,
    post_split_cmds: Option<Vec<CMD>>,
    push_this: bool,
) {
    pb.split_scope(defer_count, release_count);

    if let Some(cmds) = post_split_cmds {
        for cmd in cmds {
            pb.push(cmd);
        }
    }

    match block {
        ATermBlock::A { ref terms } => {
            for term in terms {
                add_term(pb, term, defer_count, release_count);
            }
        }
        _ => panic!(),
    }

    if push_this {
        pb.push(CMD::Push(VarAdress::Var(nms::THIS.to_string())));
    }

    pb.release_scope(defer_count, release_count);
}

fn add_object(pb: &mut ProgramBuilder, object: &AObject, parent: Option<&AObject>) {
    match &object.kind {
        AObjectType::Identity(id) => {
            match &*object._type.borrow() {
                AType::ArrayObject(..) | AType::StructObject(..) => match parent {
                    Some(_) => {
                        pb.push(CMD::Push(VarAdress::Index(id.clone())));

                        if let Some(sub) = &object.sub {
                            add_object(pb, &sub, Some(object));
                        }
                    }
                    None => {
                        pb.push(CMD::Push(VarAdress::Var(id.clone())));

                        if let Some(sub) = &object.sub {
                            add_object(pb, &sub, Some(object));
                        }
                    }
                },
                AType::FuncDefRef(_) => match &object.sub {
                    Some(sub) => add_object(pb, sub, Some(object)),
                    None => panic!(),
                },
                _ => panic!(),
            };
        }
        AObjectType::Call(acall) => {
            for arg in &acall.args {
                add_operand_block(pb, arg);
            }

            match parent {
                Some(parent) => match &*parent._type.borrow() {
                    AType::FuncDefRef(afunc) => {
                        match afunc.block {
                            AFuncBlock::Internal => {
                                pb.push(CMD::InternalOp(afunc.name.clone()));
                            }
                            AFuncBlock::InternalArray => {
                                pb.push(CMD::InternalOp(afunc.name.clone()));
                            }
                            AFuncBlock::TermsLang(_) => {
                                pb.non_indexed_refers.push((pb.len(), afunc.uid));
                                pb.push(CMD::Refer(0));
                            }
                        };
                    }
                    _ => panic!(),
                },
                _ => panic!(),
            }

            if let Some(sub) = &object.sub {
                add_object(pb, &sub, Some(object));
            }
        }
    }
}

fn add_operand_block(pb: &mut ProgramBuilder, block: &AOperandExpression) {
    match &block.value {
        AOperandExpressionValue::Dot { left, right } => {
            add_operand_block(pb, &left);
            add_object(pb, right, None);
        }
        AOperandExpressionValue::Object(aobject) => add_object(pb, aobject, None),
        AOperandExpressionValue::Literal(aliteral) => {
            pb.push(CMD::PushLit(Literal::from_aliteral(aliteral)));
        }
        AOperandExpressionValue::Create { _type, args } => match &*_type.borrow() {
            AType::ArrayObject(ref_cell) => todo!(),
            AType::StructDefRef(_t) => match _t.root {
                false => {
                    pb.push(CMD::PushObj(_t.fields.keys().map(|x| x.clone()).collect()));
                    if let Some(afunc) = _t.methods.get(nms::F_NEW) {
                        let child = AObject {
                            kind: AObjectType::Call(ACall {
                                args: args.to_vec(),
                            }),
                            sub: None,
                            _type: _type.clone(),
                            loc: crate::errors::FileLocation::None,
                        };

                        add_object(
                            pb,
                            &child,
                            Some(&AObject {
                                kind: AObjectType::Identity(nms::F_NEW.to_string()),
                                sub: None,
                                _type: Rc::new(RefCell::new(AType::FuncDefRef(afunc.clone()))),
                                loc: crate::errors::FileLocation::None,
                            }),
                        );
                    }
                }
                true => todo!(),
            },
            _ => panic!("This is the wrong type"),
        },
    }
}

fn add_term(
    pb: &mut ProgramBuilder,
    term: &ATerm,
    defer_count: &mut u32,
    release_count: &mut Vec<u32>,
) {
    match term {
        ATerm::Print { ln, value } => {
            add_operand_block(pb, value);
            pb.push(match ln {
                true => CMD::PrintLn,
                false => CMD::Print,
            });
        }
        ATerm::DeclareVar { name, value, .. } => {
            add_operand_block(pb, value);
            pb.push(CMD::Let(name.clone()));
        }
        ATerm::Return { value } => {
            add_operand_block(pb, value);
            pb.push(CMD::Defer(*defer_count));
        }
        ATerm::UpdateVar { value, var } => {
            add_operand_block(pb, value);
            add_object(pb, var, None);
            pb.push(CMD::Update);
        }
        ATerm::If {
            conditional,
            block,
            else_block,
        } => {
            add_operand_block(pb, conditional);
            pb.push(CMD::XIf);
            let else_gt = pb.push(CMD::Jump(0));
            add_block(pb, block, defer_count, release_count, None, false);
            let if_gt = pb.push(CMD::Jump(0));

            let idx = pb.len();
            if let CMD::Jump(ref mut to) = pb.tape[else_gt] {
                *to = idx;
            }

            add_block(pb, else_block, defer_count, release_count, None, false);

            let idx = pb.len();
            if let CMD::Jump(ref mut to) = pb.tape[if_gt] {
                *to = idx;
            }
        }
        ATerm::Call { value } => {
            add_operand_block(pb, value);
            pb.push(CMD::Burn);
        }
        ATerm::Loop {
            counter,
            conditional,
            block,
        } => {
            pb.split_scope(defer_count, release_count);
            pb.push(CMD::PushLit(Literal::Int(-1)));
            pb.push(CMD::Let(counter.clone()));
            let loop_start = pb.len();

            release_count.push(0);

            pb.push(CMD::PushLit(Literal::Int(1)));
            pb.push(CMD::Push(VarAdress::Var(counter.clone())));
            pb.push(CMD::InternalOp(nms::F_ADD.to_string()));
            pb.push(CMD::Push(VarAdress::Var(counter.clone())));
            pb.push(CMD::Update);
            add_operand_block(pb, conditional);
            pb.push(CMD::XIf);
            pb.non_indexed_loops.push(vec![pb.len()]);
            pb.push(CMD::Jump(1));
            add_block(pb, block, defer_count, release_count, None, false);
            pb.push(CMD::Jump(loop_start));
            release_count.pop().unwrap();

            let loop_end = pb.len();

            for jmp in pb.non_indexed_loops.pop().unwrap() {
                match pb.tape.get_mut(jmp).unwrap() {
                    CMD::Jump(jmp) => match jmp {
                        0 => *jmp = loop_start,
                        _ => *jmp = loop_end,
                    },
                    _ => panic!(),
                }
            }

            pb.release_scope(defer_count, release_count);
        }
        ATerm::Break => {
            pb.push(CMD::RelaseN(*release_count.last().unwrap()));
            let idx = pb.len();
            pb.non_indexed_loops.last_mut().unwrap().push(idx);
            pb.push(CMD::Jump(1));
        }
        ATerm::Continue => {
            pb.push(CMD::RelaseN(*release_count.last().unwrap()));
            let idx = pb.len();
            pb.non_indexed_loops.last_mut().unwrap().push(idx);
            pb.push(CMD::Jump(0));
        }
    }
}

fn add_function(pb: &mut ProgramBuilder, func: &AFunc, push_this: bool, return_this: bool) {
    pb.function_idxs.insert(func.uid, pb.len());

    if func.name == nms::F_MAIN {
        pb.main_function = pb.len();
    }

    let mut defer_count = 0;

    let mut post_split_cmds: Vec<CMD> = (&func.args)
        .into_iter()
        .map(|a| CMD::Let(a.name.clone()))
        .rev()
        .collect();

    if push_this {
        post_split_cmds.push(CMD::Let(nms::THIS.to_string()));
    }

    match &func.block {
        AFuncBlock::TermsLang(block) => add_block(
            pb,
            &block.borrow(),
            &mut defer_count,
            &mut Vec::new(),
            Some(post_split_cmds),
            return_this,
        ),
        _ => panic!(),
    }

    if let AType::StructDefRef(struct_obj) = &*func.returntype.borrow() {
        if struct_obj.name == nms::NULL && struct_obj.root && !return_this {
            pb.push(CMD::PushLit(Literal::Null));
        }
    }

    pb.push(CMD::Defer(defer_count));
}

pub fn flatten(program: &AProgram) -> FlatProgram {
    let mut pb = ProgramBuilder::new();

    for func in &program.functions {
        add_function(&mut pb, func, false, false);
    }

    for _struct in &program.structs {
        for func in _struct.methods.values() {
            add_function(&mut pb, func, true, func.name == nms::F_NEW);
        }
    }

    for (idx, func) in pb.non_indexed_refers {
        match pb.tape.get_mut(idx) {
            Some(CMD::Refer(ref mut idx)) => {
                *idx = pb.function_idxs[&func];
            }
            _ => panic!(),
        }
    }

    return FlatProgram {
        tape: pb.tape,
        start_point: pb.main_function,
    };
}
