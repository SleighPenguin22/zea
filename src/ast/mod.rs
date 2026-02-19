pub mod cformatting;
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;

use crate::ast::datatype::{TypedIdentifier, ZeaTypeIdent};
use crate::ast::statement::{StatementBlock, VarDeclAssignment};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ZeaModule {
    path: PathBuf,
    imports: ImportList,
    exports: ExportList,
    symbols: HashSet<TopLevelStatement>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TopLevelStatement {
    FuncDefinition(FuncDefinition),
    GlobalConst(GlobalConst),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct FuncDeclaration {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: ZeaTypeIdent,
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone)]
pub struct FuncDefinition {
    declaration: FuncDeclaration,
    body: StatementBlock,
}
impl PartialEq for FuncDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
    }
}
impl Eq for FuncDefinition {}
impl Hash for FuncDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.hash(state)
    }
}

/// A global Constant value, this is a variable declaration-assignment at the module level.
///
/// These declaration are compared and [`Hash`]'ed against their signature/declaration.
/// And may only be defined once.
#[derive(Debug, Clone)]
pub struct GlobalConst(VarDeclAssignment);

impl PartialEq for GlobalConst {
    fn eq(&self, other: &Self) -> bool {
        self.0.decl == other.0.decl
    }
}
impl Eq for GlobalConst {}

impl Hash for GlobalConst {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.decl.hash(state)
    }
}

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
