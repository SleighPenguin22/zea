use crate::analysis::ZeaTypeError;
use crate::ast::patterns::ZeaPattern;
use crate::ast::statement::{FuncCall, StatementBlock};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;

#[derive(Debug, Clone, PartialEq)]
pub enum ZeaExpression {
    FuncCall(FuncCall),
    Literal(Literal),
    Add(Box<ZeaExpression>, Box<ZeaExpression>),
    Sub(Box<ZeaExpression>, Box<ZeaExpression>),
    Mul(Box<ZeaExpression>, Box<ZeaExpression>),
    Div(Box<ZeaExpression>, Box<ZeaExpression>),
    Mod(Box<ZeaExpression>, Box<ZeaExpression>),
    Neg(Box<ZeaExpression>),

    LogAnd(Box<ZeaExpression>, Box<ZeaExpression>),
    LogOr(Box<ZeaExpression>, Box<ZeaExpression>),
    LogXor(Box<ZeaExpression>, Box<ZeaExpression>),
    LogNot(Box<ZeaExpression>),

    BitAnd(Box<ZeaExpression>, Box<ZeaExpression>),
    BitOr(Box<ZeaExpression>, Box<ZeaExpression>),
    BitXor(Box<ZeaExpression>, Box<ZeaExpression>),
    BitNot(Box<ZeaExpression>),
    Block(StatementBlock),

    PatternMatch(Box<ZeaExpression>, Vec<PatternMatchArm>),
    ConditionMatch(Box<ZeaExpression>, Vec<ConditionMatchArm>),
    IfThenElse(Box<ZeaExpression>, Box<ZeaExpression>, Box<ZeaExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatchArm {
    pub pattern: ZeaPattern,
    pub value: Box<ZeaExpression>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionMatchArm {
    pub condition: Box<ZeaExpression>,
    pub value: Box<ZeaExpression>,
}
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
