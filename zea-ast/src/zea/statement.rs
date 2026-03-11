#![allow(dead_code)]

use crate::zea::Type;
use crate::zea::expression::{ConditionMatch, Expression};
use crate::zea::patterns::AssignmentPattern;
use crate::zea::{Hasher, statement};
use std::hash::Hash;
use zea_macros::HashEqById;

#[derive(Debug, Clone, HashEqById)]
pub struct Statement {
    pub id: usize,
    pub(crate) kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Initialisation(Initialisation),
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    Return(Expression),
    Block(StatementBlock),
    CondMatch(Box<ConditionMatch>),
}

#[derive(Debug, Clone, HashEqById)]
pub struct Initialisation {
    pub id: usize,
    pub typ: Option<Type>,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}
#[derive(Debug, Clone, HashEqById)]
pub struct Reassignment {
    pub id: usize,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

#[derive(Debug, Clone, HashEqById)]
pub struct FunctionCall {
    pub id: usize,
    pub name: String,
    pub args: Vec<Expression>,
}
#[derive(Debug, Clone, HashEqById)]
pub struct StatementBlock {
    pub id: usize,
    pub stmts: Vec<Statement>,
}
impl IntoIterator for StatementBlock {
    type Item = Statement;
    type IntoIter = <Vec<crate::zea::statement::Statement> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.stmts.into_iter()
    }
}
impl StatementBlock {
    pub fn as_slice(&self) -> &[Statement] {
        self.stmts.as_slice()
    }
}
