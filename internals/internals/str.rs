#[derive(Debug, Clone, PartialEq)]
pub struct str(pub String);

// Implement the Display trait to allow printing str
impl std::fmt::Display for str {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement the Add trait for concatenation
impl std::ops::Add for str {
    type Output = str;

    fn add(self, other: str) -> str {
        str(format!("{}{}", self.0, other.0))
    }
}

// Implement the Sub trait (subtracting strings doesn't make sense, so let's just return the original)
impl std::ops::Sub for str {
    type Output = str;

    fn sub(self, _other: str) -> str {
        self
    }
}

// Implement the assignment operators (+= and -=)
impl std::ops::AddAssign for str {
    fn add_assign(&mut self, other: str) {
        self.0.push_str(&other.0);
    }
}

impl std::ops::SubAssign for str {
    fn sub_assign(&mut self, _other: str) {
        // No-op for subtraction
    }
}

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
