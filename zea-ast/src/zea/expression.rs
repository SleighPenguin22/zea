use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::{ExpandedBlockExpr, FunctionCall, StatementBlock};
use std::hash::{Hash, Hasher};
use zea_macros::HashEqById;

macro_rules! extended {
    ($($first:expr),+) => {{
        vec![$($first),+]
    }};
    ($($first:expr),+ ; $($rest:expr),+) => {{
        let mut v = vec![$($first),+];
        $(v.extend($rest);)+
        v
    }};

    (; $($rest:expr),+) => {{
        let mut v = Vec::new();
        $(v.extend($rest);)+
        v
    }};
}

#[derive(Debug, Clone, HashEqById)]
pub struct Expression {
    pub id: usize,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    // initial pass
    Unit,
    IntegerLiteral(u64),
    BoolLiteral(bool),
    FloatLiteral(f64),
    StringLiteral(String),
    Ident(String),
    FuncCall(FunctionCall),
    BinOpExpr(BinOp, Box<Expression>, Box<Expression>),
    UnOpExpr(UnOp, Box<Expression>),

    Block(StatementBlock),
    // PatternMatch(PatternMatch),
    // ConditionMatch(ConditionMatch),
    // IfThenElse(IfThenElse),

    // after expansion
    ExpandedBlock(Box<ExpandedBlockExpr>),
}

impl Expression {
    pub fn unit(id: usize) -> Self {
        Expression {
            id,
            kind: ExpressionKind::Unit,
        }
    }
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

#[derive(Clone, Debug, HashEqById)]
pub struct ExpandedIfThenElse {
    pub id: usize,
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Box<Expression>,
}
#[derive(Clone, Debug, HashEqById)]
pub struct PatternMatchArm {
    pub id: usize,
    pat: AssignmentPattern,
    value: Box<Expression>,
}
#[derive(Clone, Debug, HashEqById)]
pub struct ConditionMatchArm {
    pub id: usize,
    case: Box<Expression>,
    value: Box<Expression>,
}
