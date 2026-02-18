use crate::datatype::{TypedIdentifier, ZeaType};
use crate::expression::ZeaExpression;
use crate::patterns::ZeaPattern;
use std::collections::HashSet;

pub enum Statement {
    VarDecl(VarDecl),
    VarDeclAssignment(VarDeclAssignment),
    VarReassignment(VarReassignment),
    FuncCall(FuncCall),
}

pub struct VarDecl {
    pub assignee: ZeaPattern,
    pub mutable: bool,
    pub storage_qualifiers: HashSet<StorageQualifier>,
}
pub struct VarDeclAssignment {
    pub decl: VarDecl,
    pub value: ZeaExpression,
}
pub struct VarReassignment {
    pub assignee: ZeaPattern,
    pub value: ZeaExpression,
}

pub struct StatementBlock(Vec<Statement>);

pub enum StorageQualifier {
    StaticLifeTime,
}
