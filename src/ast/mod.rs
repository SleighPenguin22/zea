#![allow(dead_code)]
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;
pub mod utils;

pub use crate::ast::{
    datatype::{TypedIdentifier, ZeaTypeIdent},
    expression::{Literal, ZeaExpression},
    statement::{StatementBlock, VarInitialisation, ZeaStatement},
};

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(Debug, Default, Clone)]
pub struct ZeaModule {
    pub path: PathBuf,
    pub imports: ImportList,
    pub exports: ExportList,
    pub symbols: HashSet<TopLevelStatement>,
}
impl ZeaModule {
    pub fn find_entry_point(&self) -> Option<FuncDefinition> {
        self.iter_symbols().find_map(|symbol| match symbol {
            TopLevelStatement::FuncDefinition(f) if f.declaration.name == "main" => Some(f.clone()),
            _ => None,
        })
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &TopLevelStatement> {
        self.symbols.iter()
    }
}
#[derive(Debug, PartialEq, Clone)]
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
    pub declaration: FuncDeclaration,
    pub body: StatementBlock,
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
pub type GlobalConst = VarInitialisation;

pub type ImportList = Vec<ZeaImportStatement>;
pub type ExportList = Vec<ZeaExportStatement>;

#[derive(Debug, Clone)]
pub struct ZeaImportStatement {
    path: String,
}
#[derive(Debug, Clone)]
pub enum ZeaExportStatement {
    Func(String),
    GlobalConst(String),
}
