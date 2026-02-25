#![allow(dead_code)]
use crate::ast::Type;
use crate::ast::expression::Expression;
use crate::ast::patterns::AssignmentPattern;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ConstInitialisation(ConstInitialisation),
    VarInitialisation(VarInitialisation),
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    Return(Expression),
    VoidReturn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstInitialisation {
    pub typ: Type,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInitialisation {
    pub typ: Type,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

impl From<Reassignment> for Statement {
    fn from(var: Reassignment) -> Self {
        Statement::Reassignment(var)
    }
}

impl From<ConstInitialisation> for Statement {
    fn from(var: ConstInitialisation) -> Self {
        Statement::ConstInitialisation(var)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reassignment {
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
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

pub type StatementBlock = Vec<Statement>;
