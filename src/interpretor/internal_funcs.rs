use rand::random;

use super::{
    syntax::{self as syn},
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

pub fn add_roots(
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

pub fn subtract_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Int(left_int - right_int),
            RootObject::Float(right_float) => RootObject::Float(*left_int as f32 - right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float - *right_int as f32),
            RootObject::Float(right_float) => RootObject::Float(left_float - right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn less_roots(
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

pub fn modulo_roots(
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
    args: &Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", syn::EQUAL_FUNC),
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

pub fn less_or_equal_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Bool(left_int <= right_int),
            RootObject::Float(right_float) => RootObject::Bool(*left_int as f32 <= *right_float),
            _ => RootObject::Bool(false),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Bool(*left_float <= *right_int as f32),
            RootObject::Float(right_float) => RootObject::Bool(left_float <= right_float),
            _ => RootObject::Bool(false),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn greater_or_equal_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Bool(left_int <= right_int),
            RootObject::Float(right_float) => RootObject::Bool(*left_int as f32 >= *right_float),
            _ => RootObject::Bool(false),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Bool(*left_float >= *right_int as f32),
            RootObject::Float(right_float) => RootObject::Bool(left_float >= right_float),
            _ => RootObject::Bool(false),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn greater_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Bool(left_int > right_int),
            RootObject::Float(right_float) => RootObject::Bool(*left_int as f32 > *right_float),
            _ => RootObject::Bool(false),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Bool(*left_float > *right_int as f32),
            RootObject::Float(right_float) => RootObject::Bool(left_float > right_float),
            _ => RootObject::Bool(false),
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
        RootObject::Null => String::from(syn::NULL_STRING),
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

pub fn to_int(root: &RootObject, gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
    let int = match root {
        RootObject::String(string) => match string.parse() {
            Ok(int) => int,
            Err(_) => {
                return Err(RuntimeError(
                    format!("Cannot convert string '{}' to int", string),
                    FileLocation::None,
                ))
            }
        },
        RootObject::Int(int) => *int,
        RootObject::Float(float) => float.floor() as i32,
        RootObject::Bool(_bool) => {
            if *_bool {
                1
            } else {
                0
            }
        }
        RootObject::Null => 0,
    };

    let key = random();
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(RootObject::Int(int)),
        },
    );
    Ok(ExitMethod::ExplicitReturn(key))
}

pub fn multiply_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::String(left_string) => match right {
            RootObject::Int(right_int) => {
                RootObject::String(left_string.repeat(match (*right_int).try_into() {
                    Ok(ok) => ok,
                    Err(_) => {
                        return Err(RuntimeError(
                            format!("{} is not a valid string multiplyer.", right_int),
                            FileLocation::None,
                        ))
                    }
                }))
            }
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Int(left_int * right_int),
            RootObject::Float(right_float) => RootObject::Float((*left_int as f32) * right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float * (*right_int as f32)),
            RootObject::Float(right_float) => RootObject::Float(left_float * right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn divide_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => RootObject::Int(left_int / right_int),
            RootObject::Float(right_float) => RootObject::Float((*left_int as f32) / right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float / (*right_int as f32)),
            RootObject::Float(right_float) => RootObject::Float(left_float / right_float),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn exponent_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Int(left_int) => match right {
            RootObject::Int(right_int) => {
                RootObject::Int(left_int.pow(match (*right_int).try_into() {
                    Err(_) => {
                        return Err(RuntimeError(
                            format!("{} is not a valid power", right_int),
                            FileLocation::None,
                        ))
                    }
                    Ok(val) => val,
                }))
            }
            RootObject::Float(right_float) => {
                RootObject::Float((*left_int as f32).powf(*right_float))
            }
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        RootObject::Float(left_float) => match right {
            RootObject::Int(right_int) => RootObject::Float(left_float.powi(*right_int)),
            RootObject::Float(right_float) => RootObject::Float(left_float.powf(*right_float)),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn or_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Bool(left_bool) => match right {
            RootObject::Bool(right_bool) => RootObject::Bool(*left_bool || *right_bool),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn and_roots(
    left: &RootObject,
    right: &RootObject,
    gc: &GarbageCollector,
) -> Result<RootObject, RuntimeError> {
    Ok(match left {
        RootObject::Bool(left_bool) => match right {
            RootObject::Bool(right_bool) => RootObject::Bool(*left_bool && *right_bool),
            _ => return invalid_right_hand_side_root!(left, right, gc),
        },
        _ => return invalid_right_hand_side_root!(left, right, gc),
    })
}

pub fn std_binary_operation(
    left: &RootObject,
    gc: &mut GarbageCollector,
    args: &Vec<u32>,
    function_name: &str,
    operation: &dyn Fn(
        &RootObject,
        &RootObject,
        &GarbageCollector,
    ) -> Result<RootObject, RuntimeError>,
) -> Result<ExitMethod, RuntimeError> {
    let right_id = match args.first() {
        Some(id) => id,
        None => {
            return Err(RuntimeError(
                format!("{} must have at least one argument", function_name),
                FileLocation::None,
            ))
        }
    };
    let right_object = &gc.objects[right_id].data;
    let right = match right_object {
        DataObject::StructObject(_) => todo!(),
        DataObject::RootObject(right) => right,
        DataObject::ArrayObject(_) => todo!(),
    };
    let result = operation(&left, &right, &gc)?;
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

pub fn remove(
    id: u32,
    gc: &mut GarbageCollector,
    args: &Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let mut len = match &gc.objects[&id].data {
        DataObject::ArrayObject(arr) => arr.0.len() as i32,
        _ => todo!(),
    };

    for arg in args {
        let arg_object = &gc.objects[arg].data;
        let idx = match arg_object {
            DataObject::RootObject(RootObject::Int(idx)) => *idx,
            _ => {
                return Err(RuntimeError(
                    format!("All {} arguments must be integers.", syn::REMOVE_FUNC),
                    FileLocation::None,
                ))
            }
        };

        if idx >= len {
            return Err(RuntimeError(
                format!("Index, {}, out of range.", idx),
                FileLocation::None,
            ));
        } else if idx < 0 {
            return Err(RuntimeError(
                format!("Index, {}, out of range.", idx),
                FileLocation::None,
            ));
        } else {
            let arr = match gc.objects.get_mut(&id).unwrap().data {
                DataObject::ArrayObject(ref mut arr) => arr,
                _ => todo!(),
            };
            arr.0.remove(idx as usize);
            len -= 1;
        }
    }

    return Ok(ExitMethod::ExplicitReturn(gc.create_null_object()));
}

pub fn index(
    id: u32,
    gc: &mut GarbageCollector,
    args: &Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    let len = match &gc.objects[&id].data {
        DataObject::ArrayObject(arr) => arr.0.len() as i32,
        _ => todo!(),
    };
    let arg_object = &gc.objects[match args.first() {
        Some(obj) => obj,
        None => {
            return Err(RuntimeError(
                format!("{} must have an argument.", syn::INDEX_FUNC),
                FileLocation::None,
            ))
        }
    }]
    .data;
    let idx = match arg_object {
        DataObject::RootObject(RootObject::Int(idx)) => idx,
        _ => {
            return Err(RuntimeError(
                format!("{} argument must be integer.", syn::INDEX_FUNC),
                FileLocation::None,
            ))
        }
    };

    let result = if idx >= &len {
        return Err(RuntimeError(
            format!("Index, {}, out of range.", idx),
            FileLocation::None,
        ));
    } else if idx < &0 {
        return Err(RuntimeError(
            format!("Index, {}, out of range.", idx),
            FileLocation::None,
        ));
    } else {
        match &gc.objects[&id].data {
            DataObject::ArrayObject(arr) => arr.0[*idx as usize],
            _ => todo!(),
        }
    };

    return Ok(ExitMethod::ExplicitReturn(result));
}

pub fn append(
    id: u32,
    gc: &mut GarbageCollector,
    args: &Vec<u32>,
) -> Result<ExitMethod, RuntimeError> {
    for arg in args {
        gc.objects.get_mut(arg).unwrap().ref_count += 1;
        let arr = match gc.objects.get_mut(&id).unwrap().data {
            DataObject::ArrayObject(ref mut arr) => arr,
            _ => todo!(),
        };
        arr.0.push(*arg);
    }

    return Ok(ExitMethod::ExplicitReturn(gc.create_null_object()));
}

pub fn len(id: u32, gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
    let key = random();

    let len = match &gc.objects[&id].data {
        DataObject::ArrayObject(arr) => arr.0.len() as i32,
        _ => todo!(),
    };
    gc.objects.insert(
        key,
        DataCase {
            ref_count: 0,
            data: DataObject::RootObject(RootObject::Int(len)),
        },
    );

    return Ok(ExitMethod::ExplicitReturn(key));
}
