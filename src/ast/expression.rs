use crate::analysis::ZeaTypeError;
use crate::ast::patterns::AssignmentPattern;
use crate::ast::statement::{FunctionCall, StatementBlock};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    FuncCall(FunctionCall),
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

    PatternMatch(Box<Expression>, Vec<PatternMatchArm>),
    ConditionMatch(Vec<ConditionMatchArm>),
    IfThenElse(Box<Expression>, Box<Expression>, Box<Expression>),
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
        if let (Self::Float(a), Self::Float(b)) = (self, other) {
            if a.is_nan() && b.is_nan() {
                return true;
            }
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
            Self::Float(f) if f.is_nan() => state.write(&*f64::NAN.to_ne_bytes().as_ref()),
            Self::Float(f) => state.write(&*f.to_ne_bytes().as_ref()),
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
