use crate::c::datatype::{DerefAssignee, Type};
use crate::c::expression::Expression;
use crate::c::FunctionCall;

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    pub typ: Type,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Initialisation {
    pub typ: Type,
    pub name: String,
    pub value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    pub condition: Expression,
    pub true_branch: StatementBlock,
    pub false_branch: StatementBlock,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub condition: Expression,
    pub body: StatementBlock,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ElseBlock {
    body: StatementBlock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatementBlock(pub Vec<Statement>);
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    VariableInitialisation(Initialisation),
    VariableReassignment(Reassignment),
    Return(Expression),
    ReturnVoid,
    FunctionCall(FunctionCall),
    Reassignment(Reassignment),
    DerefAssignment(DerefReassignment),
    IfBlock(IfBlock),
    ElseBlock(ElseBlock),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Reassignment {
    assignee: String,
    value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefReassignment {
    assignee: DerefAssignee,
    value: Expression,
}
