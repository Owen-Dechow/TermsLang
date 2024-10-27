use super::{this, BlockExit, Data, DataScope, FileLocation, GlobalData, RootValue, StructObject};
use crate::{
    active_parser::{names as nm, AFunc, AType},
    ainterpretor::Root,
    errors::RuntimeError,
    rc_ref,
};
use std::{cell::RefCell, rc::Rc};

macro_rules! int {
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
                    RootValue::Int(int) => int,
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }};
}

pub fn interpret_function(
    func: &AFunc,
    ds: Option<&DataScope>,
    gd: &GlobalData,
    args: &[Rc<RefCell<Data>>],
) -> Result<BlockExit, RuntimeError> {
    let arr = match *this(ds).borrow() {
        Data::ArrayObject(ref array) => array.0.clone(),
        _ => panic!(),
    };

    match func.name.as_str() {
        nm::F_INDEX => index(arr, int!(args[0])),
        nm::F_APPEND => append(arr, args[0].clone(), gd),
        nm::F_REMOVE => remove(arr, int!(args[0])),
        nm::F_LEN => length(arr, gd),
        _ => panic!("{} is not implimented for arrays.", func.name),
    }
}

pub fn index(
    array: Rc<RefCell<Vec<Rc<RefCell<Data>>>>>,
    idx: i32,
) -> Result<BlockExit, RuntimeError> {
    Ok(BlockExit::Explicit(array.borrow()[idx as usize].clone()))
}

pub fn append(
    array: Rc<RefCell<Vec<Rc<RefCell<Data>>>>>,
    data: Rc<RefCell<Data>>,
    gd: &GlobalData,
) -> Result<BlockExit, RuntimeError> {
    array.borrow_mut().push(data);
    return Ok(BlockExit::Explicit(gd.null()));
}

pub fn remove(
    array: Rc<RefCell<Vec<Rc<RefCell<Data>>>>>,
    idx: i32,
) -> Result<BlockExit, RuntimeError> {
    let data = array.borrow_mut().remove(idx as usize);
    Ok(BlockExit::Explicit(data))
}
pub fn length(
    array: Rc<RefCell<Vec<Rc<RefCell<Data>>>>>,
    gd: &GlobalData,
) -> Result<BlockExit, RuntimeError> {
    Ok(BlockExit::Explicit(rc_ref!(Data::StructObject(
        StructObject::Root(Root {
            _type: AType::from_astruct(gd.root_types.int_type.clone())
                .borrow()
                .to_type_instance(),
            value: RootValue::Int(array.borrow().len() as i32),
        }),
    ))))
}
