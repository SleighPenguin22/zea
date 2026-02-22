use crate::ast::ZeaExpression;
use crate::ast::statement::*;
use crate::codegen::CodegenResult;
use crate::codegen::expr::CExpr;
use crate::codegen::lowering::{LoweredVarDecl, LoweredVarDeclAssignment};

pub trait CStmt {
    fn c_stmt(&self) -> CodegenResult<String>;

    fn insert_in_template(&self, template: impl Fn(String) -> String) -> CodegenResult<String> {
        Ok(template(self.c_stmt()?))
    }
}

impl CStmt for ZeaStatement {
    fn c_stmt(&self) -> CodegenResult<String> {
        match self {
            ZeaStatement::ReturnValue(r) => Ok(format!("return {};", r.c_expr()?)),
            ZeaStatement::ReturnVoid => Ok(String::from("return;")),
            _ => todo!("remaining statement formatting, namely {:?}", self),
        }
    }
}

impl CStmt for LoweredVarDeclAssignment {
    fn c_stmt(&self) -> CodegenResult<String> {
        Ok(format!(
            "{}{} {} = {};",
            self.format_mut_qualifier(),
            self.format_type_name(),
            self.format_assignee(),
            self.format_value()?
        ))
    }
}

impl CStmt for LoweredVarDecl {
    fn c_stmt(&self) -> CodegenResult<String> {
        Ok(format!(
            "{}{} {};",
            self.format_mut_qualifier(),
            self.format_type_name(),
            self.format_assignee()
        ))
    }
}
