use crate::ast::patterns::ZeaPattern;

pub enum LoweringError {
    InvalidPattern(ZeaPattern),
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                LoweringError::InvalidPattern(p) => format!("InvalidPattern: {:?}", p),
            }
        )
    }
}
