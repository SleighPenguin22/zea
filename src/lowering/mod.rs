use crate::ast::datatype::TupleSignature;
use crate::ast::expression::PatternMatchArm;
use crate::ast::statement::VarInitialisation;
use crate::ast::{
    ConstInitialisation, Expression, StatementBlock, StructDefinition, Type, TypedIdentifier,
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredVarInitialisation {
    pub typ: Type,
    pub assignee: String,
    pub value: Expression,
}

pub trait DesugarDestructuring {
    type Output;
    fn desugar_destructuring(&mut self) -> LoweringResult<Self::Output>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredConstInitialisation {
    pub temporary: ConstInitialisation,
    pub unpacked_assignments: Vec<ConstInitialisation>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredVarInitialisation {
    pub temporary: ConstInitialisation,
    pub unpacked_assignments: Vec<VarInitialisation>,
}

pub trait DesugarMatchExpression {
    fn desugar_match_arm(arm: PatternMatchArm) -> LoweringResult<StatementBlock>;
    fn desugar_match_expression(&self) -> LoweringResult<StatementBlock> {}
}


pub struct TupleNamer {
    current_id: usize,
    cache: HashSet<StructDefinition>,
}

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}

impl TupleNamer {
    pub fn new() -> Self {
        Self {
            current_id: 0,
            cache: HashSet::new(),
        }
    }

    pub fn name_tuple(&mut self, tuple: TupleSignature) -> StructDefinition {
        todo!()
    }

    pub fn has_named_tuple(tuple: &TupleSignature) -> bool {
        todo!()
    }
}
