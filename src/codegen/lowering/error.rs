use crate::ast::patterns::ZeaPattern;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoweringError {
    #[error("invalid pattern: {0}")]
    InvalidPattern(ZeaPattern),
}
