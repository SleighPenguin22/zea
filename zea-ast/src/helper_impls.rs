pub trait ASTStructuralEq {
    fn structural_eq(&self, other: &Self) -> bool;
}
impl ASTStructuralEq for String {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ASTStructuralEq for &str {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ASTStructuralEq for bool {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ASTStructuralEq for f64 {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ASTStructuralEq for u64 {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ASTStructuralEq for str {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}


impl<T> ASTStructuralEq for Vec<T>
where
    T: ASTStructuralEq,
{
    fn structural_eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(a, b)| a.structural_eq(b))
    }
}

impl<T> ASTStructuralEq for Box<T>
where
    T: ASTStructuralEq,
{
    fn structural_eq(&self, other: &Self) -> bool {
        self.as_ref().structural_eq(other.as_ref())
    }
}

impl ASTStructuralEq for usize {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
