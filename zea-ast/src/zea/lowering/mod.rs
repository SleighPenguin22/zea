use crate::zea::expression::{BinOp, Expression, UnOp};
use crate::zea::statement::Statement;
use crate::zea::{Type, TypedIdentifier};
use std::hash::{Hash, Hasher};
use thiserror::Error;
use zea_macros::HashEqById;

pub mod nodeexpansion;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Debug, Clone, HashEqById)]
pub struct ExpandedStatement {
    pub id: usize,
    pub kind: ExpandedStatementKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpandedStatementKind {
    Initialisation(ExpandedInitialisation),
    Reassignment(ExpandedReassignment),
    FunctionCall(LoweringFunctionCall),
    Block(ExpandedBlockExpr),
    UnitReturn,
    Return(ExpandedExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpandedReassignment {
    pub assignee: String,
    pub value: ExpandedExpression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoweringFunctionCall {
    pub name: String,
    pub arguments: Vec<ExpandedExpression>,
}

#[derive(Debug, Clone, HashEqById)]
pub struct ExpandedExpression {
    pub id: usize,
    kind: ExpandedExpressionKind,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ExpandedExpressionKind {
    Unit,
    FuncCall(Box<LoweringFunctionCall>),
    Ident(String),
    BinOpExpr(BinOp, Box<ExpandedExpression>, Box<ExpandedExpression>),
    UnOpExpr(UnOp, Box<ExpandedExpression>),
    IntegerLiteral(u64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),

    IfThenElse(
        Box<ExpandedExpression>,
        Box<ExpandedExpression>,
        Box<ExpandedExpression>,
    ),
    Block(Box<ExpandedBlockExpr>),
    CondMatch(Box<DesugaredCondMatch>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesugaredCondMatch {
    /// The label that the condmatch value gets i.e. `__cmatch0`, `__cmatch1` etc.
    /// This label must be unique to the scope of the function in which it exists
    id: usize,
    /// All of its cases, which may or may not contain a default arm.
    arms: Vec<ExpandedExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedBlockExpr {
    /// The label that the block expression has its value assigned to
    /// i.e. `__block0`, `__block1` etc.
    /// This label must be unique to the scope of the function in which it exists
    pub id: usize,
    pub statements: Vec<ExpandedStatement>,
    pub last: ExpandedExpression,
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, PartialEq, Clone)]
pub struct SimpleInitialisation {
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: ExpandedExpression,
}

impl SimpleInitialisation {
    pub fn new(typ: Option<Type>, assignee: impl Into<String>, value: ExpandedExpression) -> Self {
        Self {
            typ,
            assignee: assignee.into(),
            value,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ExpandedInitialisation {
    pub temporary: SimpleInitialisation,
    pub unpacked_assignments: Vec<ExpandedInitialisation>,
}

impl ExpandedInitialisation {
    pub(crate) fn new(
        temporary: SimpleInitialisation,
        unpacked_assignments: Vec<ExpandedInitialisation>,
    ) -> ExpandedInitialisation {
        Self {
            temporary,
            unpacked_assignments,
        }
    }

    pub fn simple(typ: Option<Type>, assignee: String, value: ExpandedExpression) -> Self {
        Self {
            temporary: SimpleInitialisation {
                typ,
                assignee,
                value,
            },
            unpacked_assignments: vec![],
        }
    }
}

impl From<SimpleInitialisation> for ExpandedInitialisation {
    fn from(value: SimpleInitialisation) -> Self {
        Self {
            temporary: value,
            unpacked_assignments: vec![],
        }
    }
}

#[derive(Debug, Clone, HashEqById)]
pub struct ExpandedFunction {
    id: usize,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: Vec<ExpandedStatement>,
}
