#![allow(dead_code)]
use crate::ast::ZeaTypeIdent;
use crate::ast::expression::ZeaExpression;
use crate::ast::patterns::ZeaPattern;

#[derive(Debug, Clone, PartialEq)]
pub enum ZeaStatement {
    VarInitialisation(VarInitialisation),
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

#[derive(Debug, Clone, PartialEq)]
pub struct VarInitialisation {
    pub decl: VarDecl,
    pub value: ZeaExpression,
}

impl From<VarReassignment> for ZeaStatement {
    fn from(var: VarReassignment) -> Self {
        ZeaStatement::VarReassignment(var)
    }
}

impl From<VarInitialisation> for ZeaStatement {
    fn from(var: VarInitialisation) -> Self {
        ZeaStatement::VarInitialisation(var)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarReassignment {
    pub assignee: ZeaPattern,
    pub value: ZeaExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncCall {
    pub name: String,
    pub args: Vec<ZeaExpression>,
}

impl From<FuncCall> for ZeaStatement {
    fn from(func: FuncCall) -> Self {
        ZeaStatement::FuncCall(func)
    }
}

impl From<FuncCall> for ZeaExpression {
    fn from(func: FuncCall) -> Self {
        ZeaExpression::FuncCall(func)
    }
}

pub type StatementBlock = Vec<ZeaStatement>;
