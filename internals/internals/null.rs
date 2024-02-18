pub struct null;

// Display trait
impl std::fmt::Display for null {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "null")
    }
}
