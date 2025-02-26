mod data;
mod debugger;
use std::io::stdin;

use crate::active_parser::names as nms;
use crate::errors::{FileLocation, RuntimeError};
use crate::flat_ir::{FlatProgram, VarAdress, CMD};
use data::Data;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Custom(FxHashMap<usize, Value>),
    Array(Vec<Value>),
    Null,
    Ptr(usize),
}
impl Value {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn string<'a>(&'a self, runner: &'a Runner) -> &'a String {
        match self {
            Value::Str(string) => string,
            Value::Ptr(to) => runner.data[to].0.string(runner),
            _ => panic!(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn bool<'a>(&'a self, runner: &'a Runner) -> &'a bool {
        match self {
            Value::Bool(b) => b,
            Value::Ptr(to) => runner.data[to].0.bool(runner),
            _ => panic!(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn int<'a>(&'a self, runner: &'a Runner) -> &'a i32 {
        match self {
            Value::Int(i) => i,
            Value::Ptr(to) => &runner.data[to].0.int(runner),
            _ => panic!(),
        }
    }
}

struct GlobalCounter(usize);
impl GlobalCounter {
    fn new() -> Self {
        Self(0)
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn next(&mut self) -> usize {
        self.0 += 1;
        return self.0;
    }
}

#[derive(Clone)]
pub struct Cell(Value, usize);

struct Runner<'a> {
    current_postion: usize,
    stack: Vec<Value>,
    refer_stack: Vec<usize>,
    prog: &'a FlatProgram,
    scopes: Vec<Vec<Value>>,
    data: Data,
    gc: GlobalCounter,
}
impl<'a> Runner<'a> {
    fn new(prog: &'a FlatProgram, args: &Vec<String>) -> Self {
        let data = Data::default();
        let gc = GlobalCounter::new();
        let args = args.into_iter().map(|x| Value::Str(x.clone())).collect();
        let mut scopes = Vec::new();
        scopes.resize(prog.n_scopes, Vec::new());

        Self {
            current_postion: prog.start_point,
            stack: vec![Value::Array(args)],
            refer_stack: Vec::new(),
            scopes,
            prog,
            gc,
            data,
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn stack_pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(s) => s,
            None => panic!("Stack should not be empty"),
        }
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        while !self.run_command()? {}
        return Ok(());
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn get_var(&self, var: usize) -> &Value {
        self.scopes[var].last().unwrap()
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn set_var(&mut self, var: &[usize], new: Value) {
        let scope = self.scopes.get_mut(var[0]).unwrap();

        let to = match scope.last_mut().unwrap() {
            Value::Custom(hash_map) => {
                hash_map.insert(var[1].clone(), new);
                return;
            }
            Value::Ptr(to) => match var.len() {
                1 => {
                    scope.pop();
                    scope.push(new);
                    return;
                }
                _ => *to,
            },
            _ => {
                scope.pop();
                scope.push(new);
                return;
            }
        };

        self.set_var_mut_on_obj(to, &var[1..], new);
    }

    fn set_var_mut_on_obj(&mut self, obj: usize, var: &[usize], new: Value) {
        let obj = &mut self.data.get_mut(&obj).0;

        let to = match obj {
            Value::Custom(ref mut hash_map) => match hash_map.get_mut(&var[0]).unwrap() {
                Value::Ptr(to) => match var.len() {
                    1 => {
                        hash_map.insert(var[0].clone(), new);
                        return;
                    }
                    _ => *to,
                },
                _ => {
                    hash_map.insert(var[0].clone(), new);
                    return;
                }
            },
            _ => panic!(),
        };

        self.set_var_mut_on_obj(to, &var[1..], new);
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn reduct(&self, v: &'a Value) -> &Value {
        match &v {
            Value::Ptr(to) => &self.data[to].0,
            _ => v,
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn mut_reduct<'b>(&'b mut self, v: &'b mut Value) -> &'b mut Value {
        match &v {
            Value::Ptr(to) => &mut self.data.get_mut(to).0,
            _ => v,
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn internal_op(&mut self, op: &str, loc: &FileLocation) -> Result<(), RuntimeError> {
        let val = match op {
            nms::F_LTEQ => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Bool(i1 <= i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Bool(f1 <= f2),
                    _ => panic!(),
                }
            }
            nms::F_LT => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Bool(i1 < i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Bool(f1 < f2),
                    _ => panic!(),
                }
            }
            nms::F_SUB => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1 - i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1 - f2),
                    _ => panic!(),
                }
            }
            nms::F_ADD => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1 + i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1 + f2),
                    (Value::Str(s2), Value::Str(s1)) => Value::Str(s1.clone() + &s2),
                    _ => panic!(),
                }
            }
            nms::F_EQ => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Bool(i1 == i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Bool(f1 == f2),
                    (Value::Str(s2), Value::Str(s1)) => Value::Bool(s1 == s2),
                    (Value::Bool(b2), Value::Bool(b1)) => Value::Bool(b2 == b1),
                    (Value::Null, Value::Null) => Value::Bool(true),
                    _ => Value::Bool(false),
                }
            }
            nms::F_MOD => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1 % i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1 % f2),
                    (Value::Str(s2), Value::Str(s1)) => Value::Str(s1.replace("%", s2)),
                    _ => panic!(),
                }
            }
            nms::F_STRING => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => Value::Str(s.clone()),
                    Value::Int(i) => Value::Str(i.to_string()),
                    Value::Float(f) => Value::Str(f.to_string()),
                    Value::Bool(b) => Value::Str(b.to_string()),
                    Value::Null => Value::Str(String::from(nms::NULL)),
                    _ => panic!(),
                }
            }
            nms::F_BOOL => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => Value::Bool(!s.is_empty()),
                    Value::Int(i) => Value::Bool(*i != 0),
                    Value::Float(f) => Value::Bool(*f != 0.0),
                    Value::Bool(b) => Value::Bool(*b),
                    Value::Null => Value::Bool(false),
                    _ => panic!(),
                }
            }
            nms::F_INT => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => Value::Int(match s.parse() {
                        Ok(i) => i,
                        Err(_) => {
                            return Err(RuntimeError(
                                format!("\"{}\" can not be converted into an integer.", s),
                                loc.clone(),
                            ))
                        }
                    }),
                    Value::Int(i) => Value::Int(*i),
                    Value::Float(f) => Value::Int(f.round() as i32),
                    Value::Bool(b) => Value::Int((*b).try_into().unwrap()),
                    Value::Null => Value::Int(0),
                    _ => panic!(),
                }
            }
            nms::F_FLOAT => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => Value::Float(match s.parse() {
                        Ok(i) => i,
                        Err(_) => {
                            return Err(RuntimeError(
                                format!("\"{}\" can not be converted into a float.", s),
                                loc.clone(),
                            ))
                        }
                    }),
                    Value::Int(i) => Value::Float(*i as f32),
                    Value::Float(f) => Value::Float(*f),
                    Value::Bool(b) => Value::Float((*b).try_into().unwrap()),
                    Value::Null => Value::Float(0.0),
                    _ => panic!(),
                }
            }
            nms::F_NEW => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);
                match aa {
                    Value::Str(s) => Value::Str(s.clone()),
                    Value::Int(i) => Value::Int(*i),
                    Value::Float(f) => Value::Float(*f),
                    Value::Bool(b) => Value::Bool(*b),
                    Value::Null => Value::Null,
                    _ => panic!(),
                }
            }
            nms::F_MULT => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1 * i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1 * f2),
                    _ => panic!(),
                }
            }
            nms::F_DIV => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1 / i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1 / f2),
                    _ => panic!(),
                }
            }
            nms::F_EXP => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Int(i1.pow(match (*i2).try_into() {
                        Ok(i2) => i2,
                        Err(_) =>  return Err(RuntimeError(format!("Right side of int exponent must be positive integer. Found {i2}. Convert to float to avoid this."), loc.clone())),
                    })),
                    (Value::Float(f2), Value::Float(f1)) => Value::Float(f1.powf(*f2)),
                    _ => panic!(),
                }
            }
            nms::F_GT => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Bool(i1 > i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Bool(f1 > f2),
                    _ => panic!(),
                }
            }
            nms::F_GTEQ => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Int(i2), Value::Int(i1)) => Value::Bool(i1 >= i2),
                    (Value::Float(f2), Value::Float(f1)) => Value::Bool(f1 >= f2),
                    _ => panic!(),
                }
            }
            nms::F_NOT => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);

                match aa {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => panic!(),
                }
            }
            nms::F_AND => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Bool(b1), Value::Bool(b2)) => Value::Bool(*b1 && *b2),
                    _ => panic!(),
                }
            }
            nms::F_OR => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match (aa, bb) {
                    (Value::Bool(b1), Value::Bool(b2)) => Value::Bool(*b1 || *b2),
                    _ => panic!(),
                }
            }
            nms::F_LEN => {
                let a = self.stack_pop();
                let aa = self.reduct(&a);

                match aa {
                    Value::Str(s) => Value::Int(s.len().try_into().unwrap()),
                    Value::Array(a) => Value::Int(a.len().try_into().unwrap()),
                    _ => panic!(),
                }
            }
            nms::F_INDEX => {
                let a = self.stack_pop();
                let b = self.stack_pop();
                let aa = self.reduct(&a);
                let bb = self.reduct(&b);

                match bb {
                    Value::Array(arr) => {
                        let idx = *aa.int(self);
                        let arr_len = arr.len() as i32;

                        if idx < 0 || idx + 1 > arr_len {
                            if arr_len > 0 {
                                return Err(RuntimeError(
                                    format!(
                                        "{} is out of the range of the array. Array range is [0, {}].",
                                        idx,
                                        arr_len - 1
                                    ),
                                    loc.clone(),
                                ));
                            } else {
                                return Err(RuntimeError(
                                    format!(
                                        "{} is out of the range of the array. Array is empty.",
                                        idx,
                                    ),
                                    loc.clone(),
                                ));
                            }
                        }

                        arr[idx as usize].clone()
                    }
                    _ => panic!(),
                }
            }
            nms::F_APPEND => {
                let key = self.gc.next();
                let a = self.stack_pop();
                let mut b = self.stack_pop();
                let aa = self.reduct(&a).clone();
                self.data.insert(key, Cell(aa.clone(), 1));

                let bb = self.mut_reduct(&mut b);

                match bb {
                    Value::Array(arr) => arr.push(Value::Ptr(key)),
                    _ => panic!(),
                };

                Value::Null
            }
            nms::F_REMOVE => {
                let a = self.stack_pop();
                let idx = *self.reduct(&a).clone().int(self);

                let mut b = self.stack_pop();
                let bb = self.mut_reduct(&mut b);

                match bb {
                    Value::Array(arr) => {
                        let arr_len = arr.len() as i32;

                        if idx < 0 || idx + 1 > arr_len {
                            if arr_len > 0 {
                                return Err(RuntimeError(
                                    format!(
                                        "{} is out of the range of the array. Array range is [0, {}].",
                                        idx,
                                        arr_len - 1
                                    ),
                                    loc.clone(),
                                ));
                            } else {
                                return Err(RuntimeError(
                                    format!(
                                        "{} is out of the range of the array. Array is empty.",
                                        idx,
                                    ),
                                    loc.clone(),
                                ));
                            }
                        }

                        arr.remove(idx.try_into().unwrap());

                        Value::Null
                    }
                    _ => panic!(),
                }
            }
            nms::F_READLN => {
                let mut s = String::new();
                if let Err(err) = stdin().read_line(&mut s) {
                    return Err(RuntimeError(
                        format!("Could not read: {}", err),
                        loc.clone(),
                    ));
                }
                Value::Str(s)
            }
            _ => panic!(),
        };

        self.stack.push(val);
        return Ok(());
    }

    fn release_complex(&mut self, value: Value, reserve: &usize) {
        match value {
            Value::Custom(hash_map) => {
                for val in hash_map.values() {
                    if let Value::Ptr(key) = val {
                        let data = self.data.get_mut(&key);
                        data.1 -= 1;

                        if data.1 == 0 {
                            let val = self.data.remove(&key).0;

                            if key == reserve {
                                *self.stack.last_mut().unwrap() = val;
                            } else {
                                if let Value::Array(..) | Value::Custom(..) = val {
                                    self.release_complex(val, &reserve);
                                }
                            }
                        }
                    } else if let Value::Custom(..) | Value::Array(..) = val {
                        self.release_complex(val.clone(), reserve);
                    }
                }
            }
            Value::Array(values) => {
                for val in &values {
                    if let Value::Ptr(key) = val {
                        let data = self.data.get_mut(&key);
                        data.1 -= 1;

                        if data.1 == 0 {
                            let val = self.data.remove(&key).0;

                            if key == reserve {
                                *self.stack.last_mut().unwrap() = val;
                            } else {
                                if let Value::Array(..) | Value::Custom(..) = val {
                                    self.release_complex(val, &reserve);
                                }
                            }
                        }
                    } else if let Value::Custom(..) | Value::Array(..) = val {
                        self.release_complex(val.clone(), reserve);
                    }
                }
            }
            Value::Ptr(_) => todo!(),
            _ => panic!(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn release(&mut self, nms: &Vec<usize>) {
        let reserve = match self.stack.last() {
            Some(Value::Ptr(to)) => *to,
            _ => 0,
        };

        for nm in nms {
            let key = self.scopes.get_mut(*nm).unwrap().pop().unwrap();
            if let Value::Ptr(key) = key {
                let data = self.data.get_mut(&key);
                data.1 -= 1;

                if data.1 == 0 {
                    let val = self.data.remove(&key).0;

                    if key == reserve {
                        *self.stack.last_mut().unwrap() = val;
                    } else {
                        if let Value::Array(..) | Value::Custom(..) = val {
                            self.release_complex(val, &reserve);
                        }
                    }
                }
            } else if let Value::Custom(..) | Value::Array(..) = key {
                self.release_complex(key, &reserve);
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn run_command(&mut self) -> Result<bool, RuntimeError> {
        let cmd = &self.prog.tape[self.current_postion];

        match cmd {
            CMD::SplitScope => {
                self.current_postion += 1;
            }
            CMD::Release(nms) => {
                self.release(nms);
                self.current_postion += 1;
            }
            CMD::Defer => {
                let refer = match self.refer_stack.pop() {
                    Some(refer) => refer,
                    None => return Ok(true),
                };

                self.current_postion = refer + 1;
            }
            CMD::Jump(idx) => self.current_postion = *idx,
            CMD::Push(var_address) => match var_address {
                VarAdress::Index(val) => {
                    match self.stack_pop() {
                        Value::Ptr(to) => match &self.data[&to].0 {
                            Value::Custom(hash_map) => self.stack.push(hash_map[val].clone()),
                            _ => panic!(),
                        },
                        Value::Custom(map) => self.stack.push(map[val].clone()),
                        _ => panic!(),
                    };

                    self.current_postion += 1;
                }
                VarAdress::Var(var) => {
                    let v = self.get_var(*var);
                    self.stack.push(v.clone());
                    self.current_postion += 1;
                }
            },
            CMD::Print => {
                let v = self.stack_pop();
                print!("{}", v.string(self));
                self.current_postion += 1;
            }
            CMD::PrintLn => {
                let v = self.stack_pop();
                println!("{}", v.string(self));
                self.current_postion += 1;
            }
            CMD::Let(n) => {
                let v = self.stack_pop();
                if let Value::Ptr(to) = v {
                    self.data.get_mut(&to).1 += 1;
                }

                match self.scopes.get_mut(*n) {
                    Some(lst) => lst.push(v),
                    None => {
                        self.scopes.insert(*n, vec![v]);
                    }
                }

                self.current_postion += 1;
            }
            CMD::XIf => match self.stack_pop().bool(self) {
                true => self.current_postion += 2,
                false => self.current_postion += 1,
            },
            CMD::Refer(idx) => {
                self.refer_stack.push(self.current_postion);
                self.current_postion = *idx;
            }
            CMD::InternalOp(op, loc) => {
                self.internal_op(op, loc)?;
                self.current_postion += 1;
            }
            CMD::PushLit(literal) => {
                self.stack.push(literal.clone());
                self.current_postion += 1;
            }
            CMD::PushObj(fields) => {
                let mut fmap = FxHashMap::default();
                for field in fields {
                    fmap.insert(field.clone(), Value::Null);
                }

                let key = self.gc.next();
                self.data.insert(key, Cell(Value::Custom(fmap), 0));
                self.stack.push(Value::Ptr(key));
                self.current_postion += 1;
            }
            CMD::Burn => {
                self.stack_pop();
                self.current_postion += 1;
            }
            CMD::Update(reduct) => {
                let new = self.stack_pop();

                if let Value::Ptr(ref to) = new {
                    self.data.get_mut(to).1 += 1;
                }

                self.set_var(reduct, new);
                self.current_postion += 1;
            }
            CMD::TRelease => self.current_postion += 1,
            CMD::PushVec => {
                self.stack.push(Value::Array(Vec::new()));
                self.current_postion += 1;
            }
        }

        return Ok(false);
    }
}

pub fn interpret(
    program: &FlatProgram,
    args: &Vec<String>,
    debug: bool,
) -> Result<(), RuntimeError> {
    let mut runner = Runner::new(program, args);
    match debug {
        true => debugger::Debugger::new(runner).debug(),
        false => runner.run(),
    }
}
