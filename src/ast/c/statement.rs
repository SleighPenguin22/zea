use crate::ast::c::datatype::Type;
use crate::ast::c::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    typ: Type,
    name: String,
}

pub struct Initialisation {
    typ: Type,
    name: String,
    value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    pub condition: Expression,
    pub true_branch: StatementBlock,
    pub false_branch: StatementBlock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatementBlock(Vec<Statement>);
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    VariableReassignment(Reassignment),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Reassignment {
    assignee: String,
    value: Expression,
}
