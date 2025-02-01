use super::Cell;
use rustc_hash::FxHashMap;
use std::ops::Index;

pub type Data = DataH;

pub struct DataH(FxHashMap<usize, Cell>);
impl DataH {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn default() -> Self {
        DataH(FxHashMap::default())
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn remove(&mut self, idx: &usize) -> Cell {
        self.0.remove(idx).unwrap()
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn insert(&mut self, idx: usize, value: Cell) {
        self.0.insert(idx, value);
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn get_mut(&mut self, idx: &usize) -> &mut Cell {
        self.0.get_mut(idx).unwrap()
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn get_valid_data(&self) -> Vec<(&usize, &Cell)> {
        (&self.0).into_iter().collect()
    }
}
impl Index<&usize> for DataH {
    type Output = Cell;

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn index(&self, index: &usize) -> &Self::Output {
        &self.0[index]
    }
}
