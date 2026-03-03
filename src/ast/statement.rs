#![allow(dead_code)]
use crate::ast::expression::Expression;
use crate::ast::patterns::AssignmentPattern;
use crate::ast::Type;
use crate::lowering::DesugaredCondMatch;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Initialisation(Initialisation),
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    Return(Expression),
    Block(Vec<Statement>),
    CondMatch(Box<DesugaredCondMatch>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Initialisation {
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
