#![allow(dead_code)]
use crate::ast::expression::Expression;
use crate::ast::patterns::AssignmentPattern;
use crate::ast::Type;

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
impl From<VarInitialisation> for ConstInitialisation {
    fn from(var_initialisation: VarInitialisation) -> Self {
        Self {
            typ: var_initialisation.typ,
            assignee: var_initialisation.assignee,
            value: var_initialisation.value,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct VarInitialisation {
    pub typ: Type,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

impl From<ConstInitialisation> for VarInitialisation {
    fn from(const_initialisation: ConstInitialisation) -> Self {
        Self {
            typ: const_initialisation.typ,
            assignee: const_initialisation.assignee,
            value: const_initialisation.value,
        }
    }
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
