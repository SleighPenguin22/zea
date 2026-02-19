pub mod cformatting;
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;

use crate::ast::datatype::{TypedIdentifier, ZeaTypeIdent};
use crate::ast::statement::{StatementBlock, VarDeclAssignment};

#[derive(Debug)]
pub struct ZeaModule {
    imports: ImportList,
    exports: ExportList,
    symbols: Vec<TopLevelStatement>,
}

#[derive(Debug)]
pub enum TopLevelStatement {
    FuncDefinition(FuncDefinition),
    GlobalConst(GlobalConst),
}

#[derive(Debug)]
pub struct FuncDeclaration {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: ZeaTypeIdent,
}

#[derive(Debug)]
pub struct FuncDefinition {
    declaration: FuncDeclaration,
    body: StatementBlock,
}
#[derive(Debug)]
pub struct GlobalConst(VarDeclAssignment);

pub type ImportList = Vec<ZeaImportStatement>;
pub type ExportList = Vec<ZeaExportStatement>;

#[derive(Debug)]

pub struct ZeaImportStatement {
    path: String,
}
#[derive(Debug)]

pub enum ZeaExportStatement {
    Func(String),
    GlobalConst(String),
}
