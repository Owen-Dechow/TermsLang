use super::garbage_collector::GarbageCollector;
use crate::errors::RuntimeError;
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct VariableRegistry {
    pub vars: HashMap<String, u32>,
    pub parent: Option<Rc<VariableRegistry>>,
}
impl VariableRegistry {
    pub fn resolve_string(&self, string: &String) -> Result<u32, RuntimeError> {
        match self.vars.get(string) {
            Some(resolved) => Ok(*resolved),
            None => match &self.parent {
                Some(parent) => parent.resolve_string(string),
                None => Err(RuntimeError(
                    format!("{} is not defined", string),
                    crate::errors::FileLocation::None,
                )),
            },
        }
    }

    pub fn release(&self, gc: &mut GarbageCollector) {
        for (_var, ref_id) in &self.vars {
            let data_case = gc.objects.get_mut(&ref_id).unwrap();
            data_case.ref_count -= 1;
            if data_case.ref_count == 0 {
                gc.objects.remove(&ref_id);
            }
        }
    }
    pub fn release_exclude(&self, gc: &mut GarbageCollector, key: &u32) {
        for (_var, ref_id) in &self.vars {
            let data_case = gc.objects.get_mut(&ref_id).unwrap();
            data_case.ref_count -= 1;
            if data_case.ref_count == 0 && ref_id != key {
                gc.objects.remove(&ref_id);
            }
        }
    }

    pub fn create_child(&self) -> VariableRegistry {
        VariableRegistry {
            vars: HashMap::new(),
            parent: Some(Rc::new(self.clone())),
        }
    }

    pub fn add_var(&mut self, name: &String, data_ref: u32) {
        self.vars.insert(name.clone(), data_ref);
    }
}
