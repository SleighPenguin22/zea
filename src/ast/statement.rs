#![allow(dead_code)]
use crate::ast::ZeaTypeIdent;
use crate::ast::expression::ZeaExpression;
use crate::ast::patterns::ZeaPattern;

#[derive(Debug, Clone, PartialEq)]
pub enum ZeaStatement {
    VarDecl(VarDecl),
    VarDeclAssignment(VarDeclAssignment),
    VarReassignment(VarReassignment),
    FuncCall(FuncCall),
    ReturnVoid,
    ReturnValue(ZeaExpression),
}
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct VarDecl {
    pub typ: ZeaTypeIdent,
    pub assignee: ZeaPattern,
    pub mutable: bool,
}
impl Into<ZeaStatement> for VarDecl {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDecl(self)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclAssignment {
    pub decl: VarDecl,
    pub value: ZeaExpression,
}
impl Into<ZeaStatement> for VarDeclAssignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDeclAssignment(self)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct VarReassignment {
    pub assignee: ZeaPattern,
    pub value: ZeaExpression,
}

impl Into<ZeaStatement> for VarReassignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarReassignment(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct StatementBlock(pub Vec<ZeaStatement>);

impl StatementBlock {
    pub fn iter(&self) -> impl Iterator<Item = &ZeaStatement> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = ZeaStatement> {
        self.0.into_iter()
    }
}
