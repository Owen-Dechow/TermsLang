use std::{cell::RefCell, io, rc::Rc};

use crate::{
    active_parser::{names as nm, AFunc, AType},
    errors::{FileLocation, RuntimeError},
};

use super::{rc_ref, BlockExit, Data, DataScope, GlobalData, Root, RootValue, StructObject};

macro_rules! root {
    ($data:expr) => {{
        match *$data.borrow() {
            Data::StructObject(ref struct_object) => match struct_object {
                StructObject::Root(ref root) => match root.value {
                    RootValue::Null => {
                        return Err(RuntimeError(
                            format!("{} value error.", nm::NULL),
                            FileLocation::None,
                        ))
                    }
                    _ => root,
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }};
}

macro_rules! nroot {
    ($data:expr) => {{
        match *$data.borrow() {
            Data::StructObject(ref struct_object) => match struct_object {
                StructObject::Root(ref root) => root,
                _ => panic!(),
            },
            _ => panic!(),
        }
    }};
}

fn this<'a>(ds: Option<&'a DataScope<'a>>) -> &'a Rc<RefCell<Data>> {
    match ds {
        Some(ds) => &ds.data[nm::THIS],
        None => panic!(),
    }
}

pub fn interpret_function(
    func: &AFunc,
    ds: Option<&DataScope>,
    gd: &GlobalData,
    args: &[Rc<RefCell<Data>>],
) -> Result<BlockExit, RuntimeError> {
    let out = match func.name.as_str() {
        nm::F_READLN => readln(gd),
        // Root type logic functions
        nm::F_ADD => add(this(ds), &args[0]),
        nm::F_SUB => subtract(this(ds), &args[0]),
        nm::F_MULT => multiply(this(ds), &args[0]),
        nm::F_DIV => divide(this(ds), &args[0]),
        nm::F_MOD => modulo(this(ds), &args[0]),
        nm::F_EXP => exponent(this(ds), &args[0]),
        nm::F_GT => greater_than(this(ds), &args[0]),
        nm::F_GTEQ => greater_than_or_equal(this(ds), &args[0]),
        nm::F_LT => less_than(this(ds), &args[0]),
        nm::F_LTEQ => less_than_or_equal(this(ds), &args[0]),
        nm::F_EQ => equal(this(ds), &args[0]),
        nm::F_NOT => logical_not(this(ds)),
        nm::F_AND => logical_and(this(ds), &args[0]),
        nm::F_OR => logical_or(this(ds), &args[0]),
        // Root type conversion functions
        nm::F_BOOL => to_bool(this(ds)),
        nm::F_INT => to_int(this(ds)),
        nm::F_FLOAT => to_float(this(ds)),
        nm::F_STRING => to_string(this(ds)),
        nm::F_NEW => new(this(ds), args.get(0), gd),
        _ => todo!("Function {} not yet implemented.", func.name),
    };

    Ok(BlockExit::Explicit(out?))
}

fn new(
    a: &Rc<RefCell<Data>>,
    b: Option<&Rc<RefCell<Data>>>,
    gd: &GlobalData,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    if let Some(b) = b {
        (*a).replace(Data::StructObject(StructObject::Root(nroot!(b).clone())));
    };

    return Ok(gd.null());
}

fn readln(gd: &GlobalData) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let reader = io::stdin();
    let mut buffer: String = String::new();

    reader.read_line(&mut buffer).unwrap();

    return Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: AType::from_astruct(gd.root_types.string_type.clone())
            .borrow()
            .to_type_instance(),
        value: RootValue::String(buffer),
    }))));
}

fn add(a: &Rc<RefCell<Data>>, b: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Int(a + b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Float(a + b),
            _ => panic!(),
        },
        RootValue::String(ref a_str) => match root!(b).value {
            RootValue::String(ref b_str) => RootValue::String(format!("{}{}", a_str, b_str)),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn subtract(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Int(a - b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Float(a - b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn multiply(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Int(a * b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Float(a * b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn divide(a: &Rc<RefCell<Data>>, b: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Int(a / b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Float(a / b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn modulo(a: &Rc<RefCell<Data>>, b: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match &root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Int(a % b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Float(a % b),
            _ => panic!(),
        },
        RootValue::String(a) => match &root!(b).value {
            RootValue::String(b) => RootValue::String(a.replace("%", &b)),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn exponent(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(base) => match root!(b).value {
            RootValue::Int(exp) => RootValue::Int(base.pow(exp as u32)),
            _ => panic!(),
        },
        RootValue::Float(base) => match root!(b).value {
            RootValue::Float(exp) => RootValue::Float(base.powf(exp)),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn equal(a: &Rc<RefCell<Data>>, b: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match nroot!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Bool(a == b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Bool(a == b),
            _ => panic!(),
        },
        RootValue::String(ref a_str) => match root!(b).value {
            RootValue::String(ref b_str) => RootValue::Bool(a_str == b_str),
            _ => panic!(),
        },
        RootValue::Bool(a) => match root!(b).value {
            RootValue::Bool(b) => RootValue::Bool(a == b),
            _ => panic!(),
        },
        RootValue::Null => match nroot!(b).value {
            RootValue::Null => RootValue::Bool(true),
            _ => panic!(),
        },
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: nroot!(a)._type.clone(),
        value,
    }))))
}

fn greater_than(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Bool(a > b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Bool(a > b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn greater_than_or_equal(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Bool(a >= b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Bool(a >= b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn less_than(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Bool(a < b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Bool(a < b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn less_than_or_equal(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Int(a) => match root!(b).value {
            RootValue::Int(b) => RootValue::Bool(a <= b),
            _ => panic!(),
        },
        RootValue::Float(a) => match root!(b).value {
            RootValue::Float(b) => RootValue::Bool(a <= b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn logical_not(a: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Bool(a) => RootValue::Bool(!a),
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn logical_and(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Bool(a) => match root!(b).value {
            RootValue::Bool(b) => RootValue::Bool(a && b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn logical_or(
    a: &Rc<RefCell<Data>>,
    b: &Rc<RefCell<Data>>,
) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match root!(a).value {
        RootValue::Bool(a) => match root!(b).value {
            RootValue::Bool(b) => RootValue::Bool(a || b),
            _ => panic!(),
        },
        _ => panic!(),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: root!(a)._type.clone(),
        value,
    }))))
}

fn to_bool(data: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match nroot!(data).value {
        RootValue::Bool(b) => RootValue::Bool(b),
        RootValue::Int(i) => RootValue::Bool(i != 0),
        RootValue::Float(f) => RootValue::Bool(f != 0.0),
        RootValue::String(ref s) => RootValue::Bool(!s.is_empty()),
        RootValue::Null => RootValue::Bool(false),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: nroot!(data)._type.clone(),
        value,
    }))))
}

fn to_int(data: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match nroot!(data).value {
        RootValue::Int(i) => RootValue::Int(i),
        RootValue::Float(f) => RootValue::Int(f as i32),
        RootValue::Bool(b) => RootValue::Int(if b { 1 } else { 0 }),
        RootValue::String(ref s) => RootValue::Int(s.parse::<i32>().unwrap_or(Err(RuntimeError(
            format!("Could not convert '{}' to int.", s),
            FileLocation::None,
        ))?)),
        RootValue::Null => RootValue::Int(0),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: nroot!(data)._type.clone(),
        value,
    }))))
}

fn to_float(data: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match nroot!(data).value {
        RootValue::Float(f) => RootValue::Float(f),
        RootValue::Int(i) => RootValue::Float(i as f32),
        RootValue::Bool(b) => RootValue::Float(if b { 1.0 } else { 0.0 }),
        RootValue::String(ref s) => {
            RootValue::Float(s.parse::<f32>().unwrap_or(Err(RuntimeError(
                format!("Could not convert '{}' to float.", s),
                FileLocation::None,
            ))?))
        }
        RootValue::Null => RootValue::Float(0.0),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: nroot!(data)._type.clone(),
        value,
    }))))
}

fn to_string(data: &Rc<RefCell<Data>>) -> Result<Rc<RefCell<Data>>, RuntimeError> {
    let value = match nroot!(data).value {
        RootValue::String(ref s) => RootValue::String(s.clone()),
        RootValue::Int(i) => RootValue::String(i.to_string()),
        RootValue::Float(f) => RootValue::String(f.to_string()),
        RootValue::Bool(b) => RootValue::String(b.to_string()),
        RootValue::Null => RootValue::String("null".to_string()),
    };

    Ok(rc_ref(Data::StructObject(StructObject::Root(Root {
        _type: nroot!(data)._type.clone(),
        value,
    }))))
}
