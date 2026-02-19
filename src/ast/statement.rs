use crate::ast::expression::ZeaExpression;
use crate::ast::patterns::ZeaPattern;

#[derive(Debug)]
pub enum ZeaStatement {
    VarDecl(VarDecl),
    VarDeclAssignment(VarDeclAssignment),
    VarReassignment(VarReassignment),
    FuncCall(FuncCall),
    ReturnVoid,
    ReturnValue(ZeaExpression),
}
#[derive(Debug)]
pub struct VarDecl {
    pub assignee: ZeaPattern,
    pub mutable: bool,
}
impl Into<ZeaStatement> for VarDecl {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDecl(self)
    }
}
#[derive(Debug)]
pub struct VarDeclAssignment {
    pub decl: VarDecl,
    pub value: ZeaExpression,
}
impl Into<ZeaStatement> for VarDeclAssignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarDeclAssignment(self)
    }
}
#[derive(Debug)]
pub struct VarReassignment {
    pub assignee: ZeaPattern,
    pub value: ZeaExpression,
}

impl Into<ZeaStatement> for VarReassignment {
    fn into(self) -> ZeaStatement {
        ZeaStatement::VarReassignment(self)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct StatementBlock(Vec<ZeaStatement>);
