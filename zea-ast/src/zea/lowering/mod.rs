use std::hash::{Hash, Hasher};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;
