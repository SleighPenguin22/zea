use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::{FunctionCall, StatementBlock};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unit,
    FuncCall(FunctionCall),
    Ident(String),
    Literal(Literal),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Neg(Box<Expression>),

    LogAnd(Box<Expression>, Box<Expression>),
    LogOr(Box<Expression>, Box<Expression>),
    LogXor(Box<Expression>, Box<Expression>),
    LogNot(Box<Expression>),

    BitAnd(Box<Expression>, Box<Expression>),
    BitOr(Box<Expression>, Box<Expression>),
    BitXor(Box<Expression>, Box<Expression>),
    BitNot(Box<Expression>),

    Block(StatementBlock),

    PatternMatch(PatternMatch),
    ConditionMatch(ConditionMatch),
    IfThenElse(IfThenElse),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionMatch {
    conditions: Vec<ConditionMatchArm>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatternMatch {
    subject: Box<Expression>,
    patterns: Vec<PatternMatchArm>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Option<Box<Expression>>,
}
impl IfThenElse {
    pub fn new(condition: Expression, true_case: Expression, false_case: Expression) -> IfThenElse {
        IfThenElse {
            condition: Box::new(condition),
            true_case: Box::new(true_case),
            false_case: Some(Box::new(false_case)),
        }
    }
}

pub type PatternMatchArm = (AssignmentPattern, Box<Expression>);

pub type ConditionMatchArm = (Box<Expression>, Box<Expression>);
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(u64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        if let (Self::Float(a), Self::Float(b)) = (self, other)
            && a.is_nan()
            && b.is_nan()
        {
            return true;
        }
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => unreachable!(),
        }
    }
}

impl Eq for Literal {}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Float(f) if f.is_nan() => state.write(f64::NAN.to_ne_bytes().as_ref()),
            Self::Float(f) => state.write(f.to_ne_bytes().as_ref()),
            Self::Boolean(b) => b.hash(state),
            Self::String(s) => s.hash(state),
            Self::Integer(i) => i.hash(state),
        }
    }
}

impl From<u64> for Literal {
    fn from(value: u64) -> Self {
        Literal::Integer(value)
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Literal::Float(value)
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Boolean(value)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::String(value)
    }
}
