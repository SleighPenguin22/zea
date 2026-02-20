#![allow(dead_code)]
use crate::ast::FuncDeclaration;
use crate::ast::datatype::ZeaTypeIdent;
use crate::ast::expression::ZeaExpression;
use crate::eval::ZeaValue;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct ZeaEvalError(String);

impl Debug for ZeaEvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for ZeaEvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ZeaEvalError {}

impl ZeaEvalError {
    pub fn wrap(self, wrapper: impl Fn(String) -> String) -> ZeaEvalError {
        ZeaEvalError(wrapper(self.0))
    }
}

impl Into<ZeaEvalError> for String {
    fn into(self) -> ZeaEvalError {
        ZeaEvalError(self)
    }
}

impl Into<ZeaEvalError> for &str {
    fn into(self) -> ZeaEvalError {
        ZeaEvalError(self.to_string())
    }
}

pub trait EvaluationScheme {
    fn get_annotations(&self) -> &HashMap<ZeaExpression, ZeaValue<ZeaExpression>>;

    fn lookup_var_type(&self, ident: impl Into<String>) -> Result<ZeaTypeIdent, ZeaEvalError>;

    fn lookup_func_sig(&self, ident: impl Into<String>) -> Result<FuncDeclaration, ZeaEvalError>;
}
