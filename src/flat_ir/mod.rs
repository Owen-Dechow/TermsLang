use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    active_parser::{
        names as nms, ACall, AFunc, AFuncBlock, ALiteral, AObject, AObjectType, AOperandExpression,
        AOperandExpressionValue, AProgram, ATerm, ATermBlock, AType,
    },
    errors::FileLocation,
    finterpretor::Value,
};

struct ProgramBuilder {
    tape: Vec<CMD>,
    function_idxs: HashMap<u32, usize>,
    main_function: usize,
    non_indexed_refers: Vec<(usize, u32)>,
    non_indexed_loops: Vec<Vec<usize>>,
    debug: bool,
    name_converter: VNameConverter,
}
impl ProgramBuilder {
    fn new(debug: bool) -> Self {
        ProgramBuilder {
            tape: Vec::new(),
            function_idxs: HashMap::new(),
            main_function: 0,
            non_indexed_refers: Vec::new(),
            non_indexed_loops: Vec::new(),
            debug,
            name_converter: VNameConverter::new(),
        }
    }

    fn push(&mut self, cmd: CMD) -> usize {
        self.tape.push(cmd);
        return self.len() - 1;
    }

    fn len(&self) -> usize {
        self.tape.len()
    }

    fn split_scope(
        &mut self,
        defer_count: &mut u32,
        release_count: &mut Vec<u32>,
        scopes: &mut Vec<Vec<usize>>,
        debug: bool,
    ) {
        if debug {
            self.push(CMD::SplitScope);
        }

        *defer_count += 1;

        if let Some(r) = release_count.last_mut() {
            *r += 1;
        }

        scopes.push(Vec::new());
    }

    fn add_let(&mut self, name: &str) -> usize {
        let idx = self.name_converter.convert(name);
        self.tape.push(CMD::Let(idx));
        return idx;
    }

    fn release_scope(
        &mut self,
        defer_count: &mut u32,
        release_count: &mut Vec<u32>,
        scopes: &mut Vec<Vec<usize>>,
        n: u32,
        stop_tracking: bool,
    ) {
        let mut rel = Vec::new();

        for i in 0..n {
            for x in &scopes[scopes.len() - (i + 1) as usize] {
                rel.push(x.clone());
            }
        }

        self.push(CMD::Release(rel));

        if stop_tracking {
            *defer_count -= 1;

            if let Some(r) = release_count.last_mut() {
                *r -= 1;
            }

            scopes.pop();
        }
    }
}

#[derive(Debug)]
pub enum VarAdress {
    Index(usize),
    Var(usize),
}

pub struct VNameConverter {
    name_map: HashMap<String, usize>,
    idx: usize,
}
impl VNameConverter {
    fn new() -> VNameConverter {
        VNameConverter {
            idx: 0,
            name_map: HashMap::new(),
        }
    }

    fn convert(&mut self, name: &str) -> usize {
        match self.name_map.get(name) {
            Some(idx) => *idx,
            None => {
                self.name_map.insert(name.to_string(), self.idx);
                self.idx += 1;
                return self.idx - 1;
            }
        }
    }
}

#[derive(Debug)]
pub enum CMD {
    SplitScope,
    Release(Vec<usize>),
    TRelease,
    Defer,
    Jump(usize),
    Push(VarAdress),
    Print,
    PrintLn,
    Let(usize),
    Update(Vec<usize>),
    XIf,
    Refer(usize),
    InternalOp(String, FileLocation),
    PushLit(Value),
    PushObj(Vec<usize>),
    PushVec,
    Burn,
}

pub struct FlatProgram {
    pub tape: Vec<CMD>,
    pub start_point: usize,
    pub n_scopes: usize,
}

fn add_block(
    pb: &mut ProgramBuilder,
    block: &ATermBlock,
    defer_count: &mut u32,
    release_count: &mut Vec<u32>,
    scopes: &mut Vec<Vec<usize>>,
    post_split_cmds: Option<Vec<CMD>>,
    push_this: bool,
) {
    pb.split_scope(defer_count, release_count, scopes, pb.debug);

    if let Some(cmds) = post_split_cmds {
        for cmd in cmds {
            if let CMD::Let(ref nm) = cmd {
                scopes.last_mut().unwrap().push(*nm);
            }

            pb.push(cmd);
        }
    }

    match block {
        ATermBlock::A { ref terms } => {
            for term in terms {
                add_term(pb, term, defer_count, release_count, scopes);
            }
        }
        _ => panic!(),
    }

    if push_this {
        let idx = pb.name_converter.convert(nms::THIS);
        pb.push(CMD::Push(VarAdress::Var(idx)));
    }

    if pb.debug {
        pb.push(CMD::TRelease);
    }

    pb.release_scope(defer_count, release_count, scopes, 1, true);
}

fn peek_reduct(object: &AObject, reduct: &mut Vec<usize>, pb: &mut ProgramBuilder) {
    match &object.kind {
        AObjectType::Identity(id) => {
            reduct.push(pb.name_converter.convert(id));

            if let Some(obj) = &object.sub {
                peek_reduct(&obj, reduct, pb)
            }
        }
        _ => panic!(),
    }
}

fn add_object(pb: &mut ProgramBuilder, object: &AObject, parent: Option<&AObject>) {
    match &object.kind {
        AObjectType::Identity(id) => {
            match &*object._type.borrow() {
                AType::ArrayObject(..) | AType::StructObject(..) => match parent {
                    Some(_) => {
                        let idx = pb.name_converter.convert(id);
                        pb.push(CMD::Push(VarAdress::Index(idx)));

                        if let Some(sub) = &object.sub {
                            add_object(pb, &sub, Some(object));
                        }
                    }
                    None => {
                        let idx = pb.name_converter.convert(id);
                        pb.push(CMD::Push(VarAdress::Var(idx)));

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
                                pb.push(CMD::InternalOp(afunc.name.clone(), object.loc.clone()));
                            }
                            AFuncBlock::InternalArray => {
                                pb.push(CMD::InternalOp(afunc.name.clone(), object.loc.clone()));
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
            let v = match aliteral {
                ALiteral::Int(i) => Value::Int(*i),
                ALiteral::Float(f) => Value::Float(*f),
                ALiteral::String(s) => Value::Str(s.clone()),
                ALiteral::Bool(b) => Value::Bool(*b),
            };

            pb.push(CMD::PushLit(v));
        }
        AOperandExpressionValue::Create { _type, args } => match &*_type.borrow() {
            AType::ArrayObject(..) => {
                pb.push(CMD::PushVec);
            }
            AType::StructDefRef(_t) => match _t.root {
                false => {
                    let fields = _t
                        .fields
                        .keys()
                        .map(|x| pb.name_converter.convert(x))
                        .collect();
                    pb.push(CMD::PushObj(fields));

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
    scopes: &mut Vec<Vec<usize>>,
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
            let idx = pb.add_let(name);
            scopes.last_mut().unwrap().push(idx);
        }
        ATerm::Return { value } => {
            add_operand_block(pb, value);
            pb.release_scope(defer_count, release_count, scopes, *defer_count, false);
            pb.push(CMD::Defer);
        }
        ATerm::UpdateVar { value, var } => {
            add_operand_block(pb, value);
            let mut vec = Vec::new();
            peek_reduct(var, &mut vec, pb);

            pb.push(CMD::Update(vec));
        }
        ATerm::If {
            conditional,
            block,
            else_block,
        } => {
            add_operand_block(pb, conditional);
            pb.push(CMD::XIf);
            let else_gt = pb.push(CMD::Jump(0));
            add_block(pb, block, defer_count, release_count, scopes, None, false);
            let if_gt = pb.push(CMD::Jump(0));

            let idx = pb.len();
            if let CMD::Jump(ref mut to) = pb.tape[else_gt] {
                *to = idx;
            }

            add_block(
                pb,
                else_block,
                defer_count,
                release_count,
                scopes,
                None,
                false,
            );

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
            pb.split_scope(defer_count, release_count, scopes, pb.debug);
            pb.push(CMD::PushLit(Value::Int(-1)));
            let idx = pb.add_let(&counter);
            scopes.last_mut().unwrap().push(idx);

            let loop_start = pb.len();

            release_count.push(0);

            pb.push(CMD::PushLit(Value::Int(1)));

            let idx = pb.name_converter.convert(&counter);
            pb.push(CMD::Push(VarAdress::Var(idx)));
            pb.push(CMD::InternalOp(nms::F_ADD.to_string(), FileLocation::None));

            let idx = pb.name_converter.convert(&counter);
            pb.push(CMD::Update(vec![idx]));
            add_operand_block(pb, conditional);
            pb.push(CMD::XIf);
            pb.non_indexed_loops.push(vec![pb.len()]);
            pb.push(CMD::Jump(1));
            add_block(pb, block, defer_count, release_count, scopes, None, false);
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

            pb.release_scope(defer_count, release_count, scopes, 1, true);
        }
        ATerm::Break => {
            pb.release_scope(
                defer_count,
                release_count,
                scopes,
                *release_count.last().unwrap(),
                false,
            );
            let idx = pb.len();
            pb.non_indexed_loops.last_mut().unwrap().push(idx);
            pb.push(CMD::Jump(1));
        }
        ATerm::Continue => {
            pb.release_scope(
                defer_count,
                release_count,
                scopes,
                *release_count.last().unwrap(),
                false,
            );
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
        .map(|a| CMD::Let(pb.name_converter.convert(&a.name)))
        .rev()
        .collect();

    if push_this {
        post_split_cmds.push(CMD::Let(pb.name_converter.convert(nms::THIS)));
    }

    let mut scopes = Vec::new();

    match &func.block {
        AFuncBlock::TermsLang(block) => add_block(
            pb,
            &block.borrow(),
            &mut defer_count,
            &mut Vec::new(),
            &mut scopes,
            Some(post_split_cmds),
            return_this,
        ),
        _ => panic!(),
    }

    if let AType::StructDefRef(struct_obj) = &*func.returntype.borrow() {
        if struct_obj.name == nms::NULL && struct_obj.root && !return_this {
            pb.push(CMD::PushLit(Value::Null));
        }
    }

    pb.push(CMD::Defer);
}

pub fn flatten(program: &AProgram, debug: bool) -> FlatProgram {
    let mut pb = ProgramBuilder::new(debug);

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
        n_scopes: pb.name_converter.idx + 1,
    };
}
