pub trait StructuralEq {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let _other = other;
        true
    }
}
impl StructuralEq for String {}

impl StructuralEq for &str {}

impl StructuralEq for bool {}

impl StructuralEq for f64 {}

impl StructuralEq for u64 {}

impl StructuralEq for str {}

impl<T> StructuralEq for Vec<T>
where
    T: StructuralEq,
{
    fn eq_ignore_id(&self, other: &Self) -> bool {
        (self.len() == other.len()) && self.iter().zip(other).all(|(a, b)| a.eq_ignore_id(b))
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

impl StructuralEq for usize {}

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

#[allow(unused_macros)]
macro_rules! assert_structural_eq {
    ($expected:expr, $got:expr) => {{
        use crate::visualisation::IndentPrint;
        if (&$expected).eq_ignore_id(&$got) {
            panic!(
                "expected structure did not match actual structure:\nexpected:\n{}\ngot:\n{}\n",
                $expected.indent_print(0),
                $got.indent_print(0)
            )
        }
    }};
}
pub(crate) use assert_structural_eq;
