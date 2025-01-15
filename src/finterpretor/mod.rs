mod debugger;

use crate::active_parser::names as nms;
use crate::flat_ir::{FlatProgram, Literal, VarAdress, CMD};
use rustc_hash::FxHashMap;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Custom(HashMap<String, Value>),
    Array(Vec<Value>),
    Null,
    Ptr(u32),
}
impl Value {
    #[inline(always)]
    fn string(&self) -> &String {
        match self {
            Value::Str(string) => string,
            _ => panic!(),
        }
    }

    #[inline(always)]
    fn bool(&self) -> &bool {
        match self {
            Value::Bool(b) => b,
            _ => panic!(),
        }
    }

    #[inline(always)]
    fn from_lit(lit: &Literal) -> Self {
        match lit {
            Literal::Int(i) => Value::Int(*i),
            Literal::String(i) => Value::Str(i.clone()),
            Literal::Float(i) => Value::Float(*i),
            Literal::Bool(i) => Value::Bool(*i),
        }
    }
}

struct GlobalCounter(u32);
impl GlobalCounter {
    fn new() -> Self {
        Self(0)
    }

    #[inline(always)]
    fn next(&mut self) -> u32 {
        self.0 += 1;
        return self.0;
    }
}

#[derive(Debug)]
struct Cell(Value, u32);

struct Runner<'a> {
    current_postion: usize,
    stack: Vec<Value>,
    refer_stack: Vec<usize>,
    prog: &'a FlatProgram,
    scopes: Vec<FxHashMap<&'a String, u32>>,
    data: FxHashMap<u32, Cell>,
    gc: GlobalCounter,
}
impl<'a> Runner<'a> {
    fn new(prog: &'a FlatProgram, args: &Vec<String>) -> Self {
        let args = args.into_iter().map(|x| Value::Str(x.clone())).collect();

        Self {
            current_postion: prog.start_point,
            stack: vec![Value::Array(args)],
            refer_stack: Vec::new(),
            scopes: Vec::new(),
            prog,
            gc: GlobalCounter::new(),
            data: FxHashMap::default(),
        }
    }

    #[inline(always)]
    fn stack_pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(s) => s,
            None => panic!("Stack should not be empty"),
        }
    }
    fn run(&mut self) {
        while !self.run_command() {}
    }

    #[inline(always)]
    fn get_var(&self, var: &String) -> &u32 {
        let mut idx = self.scopes.len() - 1;

        loop {
            match self.scopes[idx].get(var) {
                Some(idx) => return idx,
                None => idx -= 1,
            }
        }
    }

    #[inline(always)]
    fn reduct(&self, v: &'a Value) -> &Value {
        match &v {
            Value::Ptr(to) => &self.data[to].0,
            _ => v,
        }
    }

    #[inline(always)]
    fn internal_op(&mut self, op: &str) {
        match op {
            nms::F_LTEQ => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => self.stack.push(Value::Bool(i1 <= i2)),
                    (Value::Float(f2), Value::Float(f1)) => self.stack.push(Value::Bool(f1 <= f2)),
                    _ => panic!(),
                }
            }
            nms::F_LT => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => self.stack.push(Value::Bool(i1 < i2)),
                    (Value::Float(f2), Value::Float(f1)) => self.stack.push(Value::Bool(f1 < f2)),
                    _ => panic!(),
                }
            }
            nms::F_SUB => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => self.stack.push(Value::Int(i1 - i2)),
                    (Value::Float(f2), Value::Float(f1)) => self.stack.push(Value::Float(f1 - f2)),
                    _ => panic!(),
                };
            }
            nms::F_ADD => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => self.stack.push(Value::Int(i1 + i2)),
                    (Value::Float(f2), Value::Float(f1)) => self.stack.push(Value::Float(f1 + f2)),
                    (Value::Str(s2), Value::Str(s1)) => {
                        self.stack.push(Value::Str(s1.clone() + &s2))
                    }
                    _ => panic!(),
                }
            }
            nms::F_STRING => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => self.stack.push(Value::Str(s.clone())),
                    Value::Int(i) => self.stack.push(Value::Str(i.to_string())),
                    Value::Float(f) => self.stack.push(Value::Str(f.to_string())),
                    Value::Bool(b) => self.stack.push(Value::Str(b.to_string())),
                    Value::Null => self.stack.push(Value::Str(String::from(nms::NULL))),
                    _ => panic!(),
                }
            }
            _ => todo!("Operation Not Yet Implimented: {op}"),
        }
    }

    #[inline(always)]
    fn run_command(&mut self) -> bool {
        let cmd = &self.prog.tape[self.current_postion];

        match cmd {
            CMD::SplitScope => {
                self.scopes.push(HashMap::default());
                self.current_postion += 1;
            }
            CMD::Release => {
                for val in self.scopes.pop().unwrap().values() {
                    let cell = self.data.get_mut(val).unwrap();
                    cell.1 -= 1;

                    if cell.1 == 0 {
                        self.data.remove(val);
                    }
                }
                self.current_postion += 1;
            }
            CMD::RelaseN(n) => {
                for _ in 0..*n {
                    for val in self.scopes.pop().unwrap().values() {
                        let cell = self.data.get_mut(val).unwrap();
                        cell.1 -= 1;

                        if cell.1 == 0 {
                            self.data.remove(val);
                        }
                    }
                }

                self.current_postion += 1;
            }
            CMD::Defer(n) => {
                let preserve = match self.stack.last() {
                    Some(Value::Ptr(to)) => *to,
                    _ => 0,
                };

                let refer = match self.refer_stack.pop() {
                    Some(refer) => refer,
                    None => return true,
                };

                for _ in 0..*n {
                    for val in self.scopes.pop().unwrap().values() {
                        let cell = self.data.get_mut(val).unwrap();
                        cell.1 -= 1;

                        if cell.1 == 0 {
                            match *val == preserve {
                                true => {
                                    *self.stack.last_mut().unwrap() =
                                        self.data.remove(val).unwrap().0;
                                }
                                false => {
                                    self.data.remove(val);
                                }
                            }
                        }
                    }
                }
                self.current_postion = refer + 1;
            }
            CMD::Jump(idx) => self.current_postion = *idx,
            CMD::Push(var_adress) => match var_adress {
                VarAdress::Index(_) => todo!(),
                VarAdress::Var(var) => {
                    let v = *self.get_var(var);
                    self.stack.push(Value::Ptr(v));
                    self.current_postion += 1;
                }
            },
            CMD::Print => {
                let v = self.stack_pop();
                print!("{}", v.string());
                self.current_postion += 1;
            }
            CMD::PrintLn => {
                let v = self.stack_pop();
                println!("{}", v.string());
                self.current_postion += 1;
            }
            CMD::Let(n) => {
                let v = self.stack_pop();
                let idx = match v {
                    Value::Ptr(to) => {
                        self.data.get_mut(&to).unwrap().1 += 1;
                        to
                    }
                    _ => {
                        let idx = self.gc.next();
                        self.data.insert(idx, Cell(v, 1));
                        idx
                    }
                };

                let lst = self.scopes.last_mut().unwrap();
                lst.insert(n, idx);
                self.current_postion += 1;
            }
            CMD::XIf => match self.stack_pop().bool() {
                true => self.current_postion += 2,
                false => self.current_postion += 1,
            },
            CMD::Refer(idx) => {
                self.refer_stack.push(self.current_postion);
                self.current_postion = *idx;
            }
            CMD::InternalOp(op) => {
                self.internal_op(op);
                self.current_postion += 1;
            }
            CMD::PushLit(literal) => {
                self.stack.push(Value::from_lit(literal));
                self.current_postion += 1;
            }
            CMD::Burn => {
                self.stack_pop();
                self.current_postion -= 1;
            }
            CMD::Update => {
                match self.stack_pop() {
                    Value::Ptr(to) => self.data.get_mut(&to).unwrap().0 = self.stack_pop(),
                    _ => panic!(),
                }
                self.current_postion += 1;
            }
        }

        return false;
    }
}

pub fn interpret(program: &FlatProgram, args: &Vec<String>, debug: bool) {
    let mut runner = Runner::new(program, args);
    match debug {
        true => debugger::Debugger::new(runner).debug(),
        false => runner.run(),
    }
}
