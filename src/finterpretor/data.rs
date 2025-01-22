use super::{Cell, Value};
use rustc_hash::FxHashMap;
use std::ops::Index;

pub type Data = DataH;

pub struct DataH(FxHashMap<usize, Cell>);
impl DataH {
    #[cfg_attr(feature="inline", inline(always))]
    pub fn default() -> Self {
        DataH(FxHashMap::default())
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn remove(&mut self, idx: &usize) -> Cell {
        self.0.remove(idx).unwrap()
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn insert(&mut self, idx: usize, value: Cell) {
        self.0.insert(idx, value);
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn get_mut(&mut self, idx: &usize) -> &mut Cell {
        self.0.get_mut(idx).unwrap()
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn get_valid_data(&self) -> Vec<(&usize, &Cell)> {
        (&self.0).into_iter().collect()
    }
}
impl Index<&usize> for DataH {
    type Output = Cell;

    #[cfg_attr(feature="inline", inline(always))]
    fn index(&self, index: &usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct DataV(Vec<Cell>);
impl DataV {
    #[cfg_attr(feature="inline", inline(always))]
    pub fn default() -> Self {
        Self(Vec::new())
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn remove(&mut self, idx: &usize) -> Cell {
        std::mem::replace(&mut self.0[*idx], Cell(Value::Null, 0))
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn insert(&mut self, idx: usize, value: Cell) {
        self.0.resize((idx + 1) as usize, Cell(Value::Null, 0));
        std::mem::replace(&mut self.0[idx], value);
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn get_mut(&mut self, idx: &usize) -> &mut Cell {
        self.0.get_mut(*idx).unwrap()
    }

    #[cfg_attr(feature="inline", inline(always))]
    pub fn get_valid_data(&self) -> Vec<(usize, &Cell)> {
        let out = (&self.0).into_iter().enumerate().collect();
        return out;
    }
}
impl Index<&usize> for DataV {
    type Output = Cell;

    #[cfg_attr(feature="inline", inline(always))]
    fn index(&self, idx: &usize) -> &Self::Output {
        &self.0[*idx]
    }
}
