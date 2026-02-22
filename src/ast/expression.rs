use crate::analysis::ZeaTypeError;
use crate::ast::patterns::ZeaPattern;
use crate::ast::statement::{FuncCall, StatementBlock};
use std::fmt::{Display, Formatter};

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

impl Display for ZeaExpression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let s = match self {
            ZeaExpression::Literal(l) => format!("{:?}", l),
            ZeaExpression::Add(l, r) => format!("{} + {}", l, r),
            _ => todo!(),
        };
        f.write_str(&s)
    }
}

impl ZeaExpression {
    pub fn wrap_cascading_type_error(err: ZeaTypeError) -> ZeaTypeError {
        err.wrap(|err| format!("namely in:\n{err}\n"))
    }

    pub fn wrap_outer_type_error(&self, err: ZeaTypeError) -> ZeaTypeError {
        err.wrap(|err| format!("Type Error in expression\n{self}\n{err}\n"))
    }
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
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(u64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl Into<Literal> for u64 {
    fn into(self) -> Literal {
        Literal::Integer(self)
    }
}
impl Into<Literal> for f64 {
    fn into(self) -> Literal {
        Literal::Float(self)
    }
}
impl Into<Literal> for bool {
    fn into(self) -> Literal {
        Literal::Boolean(self)
    }
}

impl Into<Literal> for String {
    fn into(self) -> Literal {
        Literal::String(self)
    }
}
