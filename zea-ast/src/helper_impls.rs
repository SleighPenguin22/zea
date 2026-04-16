pub trait StructuralEq {
    fn structural_eq(&self, other: &Self) -> bool;
}
impl StructuralEq for String {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for &str {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for bool {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for f64 {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for u64 {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for str {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}


impl<T> StructuralEq for Vec<T>
where
    T: StructuralEq,
{
    fn structural_eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(a, b)| a.structural_eq(b))
    }
}

impl<T> StructuralEq for Box<T>
where
    T: StructuralEq,
{
    fn structural_eq(&self, other: &Self) -> bool {
        self.as_ref().structural_eq(other.as_ref())
    }
}

impl StructuralEq for usize {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

macro_rules! assert_structural_eq {
    ($expected:expr, $got:expr) => {assert!((&$expected).structural_eq(&$got))};
}
pub(crate) use assert_structural_eq;
