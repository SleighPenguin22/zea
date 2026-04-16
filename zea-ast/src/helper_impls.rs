pub trait StructuralEq {
    fn eq_ignore_id(&self, other: &Self) -> bool;
}
impl StructuralEq for String {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for &str {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl StructuralEq for bool {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for f64 {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for u64 {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl StructuralEq for str {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T> StructuralEq for Vec<T>
where
    T: StructuralEq,
{
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(a, b)| a.eq_ignore_id(b))
    }
}

impl<T> StructuralEq for Box<T>
where
    T: StructuralEq,
{
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.as_ref().eq_ignore_id(other.as_ref())
    }
}

impl StructuralEq for usize {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T> StructuralEq for Option<T>
where
    T: StructuralEq,
{
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(t1), Some(t2)) if t1.eq_ignore_id(t2) => true,
            _ => false,
        }
    }
}

macro_rules! assert_structural_eq {
    ($expected:expr, $got:expr) => {{
        use crate::visualisation::IndentPrint;
        if (&$expected).eq_ignore_id(&$got) {
            panic!("expected structure did not match actual structure:\nexpected:\n{}\ngot:\n{}\n", $expected.indent_print(0), $got.indent_print(0))
        }
    }};
}
pub(crate) use assert_structural_eq;
