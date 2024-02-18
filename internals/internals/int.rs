use crate::internals::str::str;
use std::{
    cmp::Ordering,
    ops::{
        Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
        SubAssign,
    },
};

#[derive(Debug, Copy, Clone, Eq, Ord)]
pub struct int(pub i128);

// Arithmetic operations
impl Add for int {
    type Output = int;

    fn add(self, other: int) -> int {
        int(self.0 + other.0)
    }
}

impl Sub for int {
    type Output = int;

    fn sub(self, other: int) -> int {
        int(self.0 - other.0)
    }
}

impl Mul for int {
    type Output = int;

    fn mul(self, other: int) -> int {
        int(self.0 * other.0)
    }
}

impl Div for int {
    type Output = int;

    fn div(self, other: int) -> int {
        int(self.0 / other.0)
    }
}

impl Rem for int {
    type Output = int;

    fn rem(self, other: int) -> int {
        int(self.0 % other.0)
    }
}

// Assignment operators
impl AddAssign for int {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for int {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl MulAssign for int {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl DivAssign for int {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}

impl RemAssign for int {
    fn rem_assign(&mut self, other: Self) {
        self.0 %= other.0;
    }
}

// Negation
impl Neg for int {
    type Output = int;

    fn neg(self) -> int {
        int(-self.0)
    }
}

// Comparison traits
impl PartialEq for int {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for int {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// Display trait
impl std::fmt::Display for int {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Deref and DerefMut for string-like behavior
impl Deref for int {
    type Target = i128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for int {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl int {
    pub fn __at__str(&self) -> str {
        str(self.0.to_string())
    }
}
