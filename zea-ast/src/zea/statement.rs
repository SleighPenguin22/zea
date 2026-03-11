#![allow(dead_code)]

use crate::zea::expression::{ConditionMatch, Expression};
use crate::zea::patterns::AssignmentPattern;
use crate::zea::Type;
use crate::zea::{statement, Hasher};
use std::hash::Hash;
use zea_macros::HashById;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Initialisation(Initialisation),
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    Return(Expression),
    Block(StatementBlock),
    CondMatch(Box<ConditionMatch>),
}

#[derive(Debug, Clone, PartialEq, HashById)]
pub struct Initialisation {
    pub id: usize,
    pub typ: Option<Type>,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

impl From<Reassignment> for Statement {
    fn from(var: Reassignment) -> Self {
        Statement::Reassignment(var)
    }
}

impl From<Initialisation> for Statement {
    fn from(var: Initialisation) -> Self {
        Statement::Initialisation(var)
    }
}

#[derive(Debug, Clone, PartialEq, HashById)]
pub struct Reassignment {
    pub id: usize,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, HashById)]
pub struct FunctionCall {
    pub id: usize,
    pub name: String,
    pub args: Vec<Expression>,
}

impl From<FunctionCall> for Statement {
    fn from(func: FunctionCall) -> Self {
        Statement::FunctionCall(func)
    }
}

impl From<FunctionCall> for Expression {
    fn from(func: FunctionCall) -> Self {
        Expression::FuncCall(func)
    }
}

#[derive(Debug, Clone, PartialEq, HashById)]
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
