use super::{BlockExit, Data, DataScope, GlobalData};
use crate::{active_parser::AFunc, errors::RuntimeError};
use std::{cell::RefCell, rc::Rc};

pub fn interpret_function(
    func: &AFunc,
    ds: Option<&DataScope>,
    gd: &GlobalData,
    args: &[Rc<RefCell<Data>>],
) -> Result<BlockExit, RuntimeError> {
    todo!()
}

// pub fn index(
//     array: Rc<RefCell<Data>>,
//     args: &[Rc<RefCell<Data>>],
// ) -> Result<BlockExit, RuntimeError> {
//     todo!()
// }

// pub fn length(
//     array: Rc<RefCell<Data>>,
//     args: &[Rc<RefCell<Data>>],
// ) -> Result<BlockExit, RuntimeError> {
//     todo!()
// }

// pub fn append(
//     array: Rc<RefCell<Data>>,
//     args: &[Rc<RefCell<Data>>],
// ) -> Result<BlockExit, RuntimeError> {
//     todo!()
// }

// pub fn remove(
//     array: Rc<RefCell<Data>>,
//     args: &[Rc<RefCell<Data>>],
// ) -> Result<BlockExit, RuntimeError> {
//     todo!()
// }
