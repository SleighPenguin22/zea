#![allow(dead_code)]

use crate::zea::expression::{ConditionMatch, Expression};
use crate::zea::patterns::AssignmentPattern;
use crate::zea::Type;
use std::hash::{Hash, Hasher};
use zea_macros::HashEqById;

#[derive(Debug, Clone, HashEqById)]
pub struct Statement {
    pub id: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    // initial pass
    Initialisation(Initialisation),
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    Return(Expression),
    BlockTail(Expression),
    Block(StatementBlock),
    CondMatch(Box<ConditionMatch>),

    // after expansion
    ExpandedBlock(ExpandedBlockExpr),
    ExpandedInitialisation(ExpandedInitialisation),
    SimpleInitialisation(SimpleInitialisation),
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
    pub assignee: String,
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
    pub statements: Vec<Statement>,
}

impl IntoIterator for StatementBlock {
    type Item = Statement;
    type IntoIter = <Vec<Statement> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.statements.into_iter()
    }
}
impl StatementBlock {
    pub fn as_slice(&self) -> &[Statement] {
        self.statements.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedBlockExpr {
    /// The label that the block expression has its value assigned to
    /// i.e. `__block0`, `__block1` etc.
    /// This label must be unique to the scope of the function in which it exists
    pub id: usize,
    pub statements: Vec<Statement>,
    pub last: Expression,
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, Clone, HashEqById)]
pub struct SimpleInitialisation {
    pub id: usize,
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: Expression,
}

#[derive(Debug, Clone, HashEqById)]
pub struct ExpandedInitialisation {
    pub id: usize,
    pub temporary: SimpleInitialisation,
    pub unpacked_assignments: Vec<ExpandedInitialisation>,
}
