use rand::random;

use super::{Data, DataCase, DataObject, ExitMethod, GarbageCollector, RootObject};
use crate::errors::{FileLocation, RuntimeError};
use std::io;

pub fn readln(gc: &mut GarbageCollector) -> Result<ExitMethod, RuntimeError> {
    let mut string = String::new();
    let res = io::stdin().read_line(&mut string);
    match res {
        Ok(_) => {
            let key = random();
            gc.data.insert(
                key,
                DataCase {
                    ref_count: 0,
                    data: Data::DataObject(DataObject::RootObject(RootObject::String(string))),
                },
            );
            Ok(ExitMethod::ExplicitReturn(key))
        }
        Err(err) => Err(RuntimeError(err.to_string(), FileLocation::None)),
    }
}
