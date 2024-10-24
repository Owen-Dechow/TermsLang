use std::{cell::RefCell, rc::Rc};

use crate::{active_parser::AFunc, errors::RuntimeError};

use super::{BlockExit, Data, DataScope, GlobalData};

pub fn interpret_function(
    func: &AFunc,
    ds: Option<&DataScope>,
    gd: &GlobalData,
    args: &[Rc<RefCell<Data>>],
) -> Result<BlockExit, RuntimeError> {
    todo!("Function {} not yet implimented for root type.", func.name)
}
