use crate::ast::datatype::{TypedIdentifier, ZeaTypeIdent};
use crate::ast::statement::{StatementBlock, VarDeclAssignment, Visibility};

pub struct ZeaModule {
    imports: ImportList,
    exports: ExportList,
    symbols: Vec<TopLevelStatement>,
}

pub enum TopLevelStatement {
    FuncDefinition(FuncDefinition),
    GlobalConst(GlobalConst),
}

pub struct FuncDeclaration {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: ZeaTypeIdent,
}

pub struct FuncDefinition {
    declaration: FuncDeclaration,
    body: StatementBlock,
}

pub struct GlobalConst(VarDeclAssignment);

pub type ImportList = Vec<ZeaImportStatement>;
pub type ExportList = Vec<ZeaExportStatement>;

pub struct ZeaImportStatement {
    path: String,
}

pub enum ZeaExportStatement {
    Func(String),
    GlobalConst(String),
}
