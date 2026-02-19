use crate::datatype::{TypedIdentifier, ZeaType};
use crate::expression::ZeaExpression;
use crate::patterns::ZeaPattern;
use std::collections::HashSet;

pub enum ZeaStatement {
    VarDecl(VarDecl),
    VarDeclAssignment(VarDeclAssignment),
    VarReassignment(VarReassignment),
    FuncCall(FuncCall),
    ReturnVoid,
    ReturnValue(ZeaExpression),
}

pub struct VarDecl {
    pub assignee: ZeaPattern,
    pub mutable: bool,
    pub storage_qualifiers: HashSet<StorageQualifier>,
}
impl Into<ZeaStatement> for VarDecl {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDecl(self)
    }
}

pub struct VarDeclAssignment {
    pub decl: VarDecl,
    pub value: ZeaExpression,
}
impl Into<ZeaStatement> for VarDeclAssignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDeclAssignment(self)
    }
}

pub struct VarReassignment {
    pub assignee: ZeaPattern,
    pub value: ZeaExpression,
}

impl Into<ZeaStatement> for VarReassignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarReassignment(self)
    }
}

pub struct FuncCall {
    pub name: String,
    pub args: Vec<ZeaExpression>,
}

impl Into<ZeaStatement> for FuncCall {
    fn into(self) -> ZeaStatement {
        ZeaStatement::FuncCall(self)
    }
}
impl Into<ZeaExpression> for FuncCall {
    fn into(self) -> ZeaExpression {
        ZeaExpression::FuncCall(self)
    }
}

pub struct StatementBlock(Vec<ZeaStatement>);

pub enum StorageQualifier {
    StaticLifeTime,
}

pub enum Visibilty {
    Public,
    Private,
}
