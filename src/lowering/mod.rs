pub mod codegen;

use crate::ast::expression::{ConditionMatchArm, PatternMatchArm};
use crate::ast::statement::{FunctionCall, VarInitialisation};
use crate::ast::{
    AssignmentPattern, ConstInitialisation, Expression, Literal, StatementBlock, Type,
};
use crate::lowering::codegen::CNode;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;

pub trait LowerInto<T> {
    fn lower_into(self) -> LoweringResult<T>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredVarInitialisation {
    pub typ: Type,
    pub assignee: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredConstInitialisation {
    pub typ: Type,
    pub assignee: String,
    pub value: Expression,
}

impl LowerInto<Vec<LoweredVarInitialisation>> for VarInitialisation {
    fn lower_into(self) -> LoweringResult<Vec<LoweredVarInitialisation>> {
        match self.assignee {
            AssignmentPattern::Identifier(assignee) => Ok(vec![LoweredVarInitialisation {
                assignee,
                typ: self.typ,
                value: self.value,
            }]),
            AssignmentPattern::Tuple(_) => {
                unimplemented!("remaining lowerings for assignment patterns")
            }
        }
    }
}
