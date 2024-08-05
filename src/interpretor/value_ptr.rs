use rand::random;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ptr(u32);
impl Ptr {
    pub fn new() -> Self {
        Self(random())
    }
}
