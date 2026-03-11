use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::{FunctionCall, StatementBlock};
use std::hash::{Hash, Hasher};
use zea_macros::HashEqById;

#[derive(Debug, Clone, HashEqById)]
pub struct Expression {
    id: usize,
    pub(crate) kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Unit,
    FuncCall(FunctionCall),
    Ident(String),
    BinOpExpr(BinOp, Box<Expression>, Box<Expression>),
    UnOpExpr(UnOp, Box<Expression>),

    IntegerLiteral(u64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),

    Block(StatementBlock),
    PatternMatch(PatternMatch),
    ConditionMatch(ConditionMatch),
    IfThenElse(IfThenElse),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogAnd,
    LogOr,
    LogXor,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}

#[derive(Clone, Debug, HashEqById)]
pub struct ConditionMatch {
    pub id: usize,
    conditions: Vec<ConditionMatchArm>,
}

#[derive(Clone, Debug, HashEqById)]
pub struct PatternMatch {
    pub id: usize,
    subject: Box<Expression>,
    patterns: Vec<PatternMatchArm>,
}

#[derive(Clone, Debug, HashEqById)]
pub struct IfThenElse {
    pub id: usize,
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Option<Box<Expression>>,
}
impl IfThenElse {
    pub fn new(
        id: usize,
        condition: Expression,
        true_case: Expression,
        false_case: Expression,
    ) -> IfThenElse {
        IfThenElse {
            id,
            condition: Box::new(condition),
            true_case: Box::new(true_case),
            false_case: Some(Box::new(false_case)),
        }
    }
}

pub type PatternMatchArm = (AssignmentPattern, Box<Expression>);

pub type ConditionMatchArm = (Box<Expression>, Box<Expression>);

impl From<u64> for ExpressionKind {
    fn from(value: u64) -> Self {
        ExpressionKind::IntegerLiteral(value)
    }
}

impl From<f64> for ExpressionKind {
    fn from(value: f64) -> Self {
        ExpressionKind::FloatLiteral(value)
    }
}

impl From<bool> for ExpressionKind {
    fn from(value: bool) -> Self {
        ExpressionKind::BoolLiteral(value)
    }
}

impl From<String> for ExpressionKind {
    fn from(value: String) -> Self {
        ExpressionKind::StringLiteral(value)
    }
}
