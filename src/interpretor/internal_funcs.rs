use rand::random;

use super::{
    syntax::{self, ADD_FUNC, LESS_FUNC, MODULO_FUNC},
    DataCase, DataObject, ExitMethod, GarbageCollector, RootObject, StructDef,
};
use crate::errors::{FileLocation, RuntimeError};
use std::io;

pub fn readln(gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
    let mut string = String::new();
    let res = io::stdin().read_line(&mut string);
    string.remove(string.len() - 1);
    match res {
        Ok(_) => {
            let key = random();
            gc.objects.insert(
                key,
                DataCase {
                    ref_count: 0,
                    data: DataObject::RootObject(RootObject::String(string)),
                },
            );
            Ok(ExitMethod::ExplicitReturn(key))
        }
        Err(err) => Err(RuntimeError(err.to_string(), FileLocation::None)),
    }
}

macro_rules! invalid_right_hand_side_root {
    ($left:expr, $right:expr, $gc:expr) => {
        Err(RuntimeError(
            format!(
                "Invalid right hand side type, {}, when {} is on left hand side.",
                if let StructDef::Root { name, .. } = $right.get_root_type_def($gc) {
                    name
                } else {
                    todo!()
                },
                if let StructDef::Root { name, .. } = $left.get_root_type_def($gc) {
                    name
                } else {
                    todo!()
                }
            ),
            FileLocation::None,
        ))
    };
}

pub fn add(
    left: &RootObject,
    gc: &mut GarbageCollector,
    args: Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", ADD_FUNC),
                FileLocation::None,
            ))
        }
    };
    let right_object = &gc.objects[right_id].data;
    let right: &RootObject = match right_object {
        DataObject::StructObject(_) => todo!(),
        DataObject::RootObject(right) => right,
        DataObject::ArrayObject(_) => todo!(),
    };
    let result = add_roots(left, &right, &gc)?;
    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(result),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

fn add_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::String(left_string) => match right {
            RootObject::String(right_string) => {
                RootObject::String(format!("{left_string}{right_string}"))
            }
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Int(left_int + right_int),
            RootObject::Float(right_float) => RootObject::Float(*left_int as f32 + right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float + *right_int as f32),
            RootObject::Float(right_float) => RootObject::Float(left_float + right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn less(
    left: &RootObject,
    gc: &mut GarbageCollector,
    args: Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", LESS_FUNC),
                FileLocation::None,
            ))
        }
    };
    let right_object = &gc.objects[right_id].data;
    let right: &RootObject = match right_object {
        DataObject::StructObject(_) => todo!(),
        DataObject::RootObject(right) => right,
        DataObject::ArrayObject(_) => todo!(),
    };
    let result = less_roots(left, &right, &gc)?;
    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(result),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

fn less_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Bool(left_int < right_int),
            RootObject::Float(right_float) => RootObject::Bool((*left_int as f32) < *right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Bool(*left_float < *right_int as f32),
            RootObject::Float(right_float) => RootObject::Bool(left_float < right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn to_string(root: &RootObject, gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
    let string = match root {
        RootObject::String(string) => string.to_owned(),
        RootObject::Int(int) => int.to_string(),
        RootObject::Float(float) => float.to_string(),
        RootObject::Bool(_bool) => _bool.to_string(),
        RootObject::Null => String::from(syntax::NULL_STRING),
    };

    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(RootObject::String(string)),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

pub fn modulo(
    left: &RootObject,
    gc: &mut GarbageCollector,
    args: Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", MODULO_FUNC),
                FileLocation::None,
            ))
        }
    };
    let right_object = &gc.objects[right_id].data;
    let right: &RootObject = match right_object {
        DataObject::StructObject(_) => todo!(),
        DataObject::RootObject(right) => right,
        DataObject::ArrayObject(_) => todo!(),
    };
    let result = modulo_roots(left, &right, &gc)?;
    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(result),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

fn modulo_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Int(left_int % right_int),
            RootObject::Float(right_float) => RootObject::Float(*left_int as f32 % right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float % *right_int as f32),
            RootObject::Float(right_float) => RootObject::Float(left_float % right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::String(left_string) => match right {
            RootObject::String(right_string) => {
                RootObject::String(left_string.replace("%", right_string))
            }
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn equal(
    left: &RootObject,
    gc: &mut GarbageCollector,
    args: Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", MODULO_FUNC),
                FileLocation::None,
            ))
        }
    };
    let right_object = &gc.objects[right_id].data;
    let result = match right_object {
        DataObject::StructObject(_) => RootObject::Bool(false),
        DataObject::RootObject(right) => equal_roots(left, &right)?,
        DataObject::ArrayObject(_) => RootObject::Bool(false),
    };
    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(result),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

fn equal_roots(left: &RootObject, right: &RootObject) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Bool(left_int == right_int),
            RootObject::Float(right_float) => RootObject::Bool(*left_int as f32 == *right_float),
            _ => RootObject::Bool(false),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Bool(*left_float == *right_int as f32),
            RootObject::Float(right_float) => RootObject::Bool(left_float == right_float),
            _ => RootObject::Bool(false),
        },
        RootObject::String(left_string) => match right {
            RootObject::String(right_string) => RootObject::Bool(left_string == right_string),
            _ => RootObject::Bool(false),
        },
        RootObject::Bool(left_bool) => match right {
            RootObject::Bool(right_bool) => RootObject::Bool(left_bool == right_bool),
            _ => RootObject::Bool(false),
        },
        RootObject::Null => match right {
            RootObject::Null => RootObject::Bool(true),
            _ => RootObject::Bool(false),
        },
    })
}
