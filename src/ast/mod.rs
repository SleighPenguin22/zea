#![allow(dead_code)]
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;

pub use crate::ast::datatype::{TypedIdentifier, ZeaTypeIdent};
pub use crate::ast::expression::{Literal, ZeaExpression};
pub use crate::ast::statement::{StatementBlock, VarDeclAssignment, ZeaStatement};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct ZeaModule {
    pub path: PathBuf,
    pub imports: ImportList,
    pub exports: ExportList,
    pub symbols: HashSet<TopLevelStatement>,
}
impl ZeaModule {
    pub(crate) fn find_entry_point(&self) -> Option<FuncDefinition> {
        self.iter_symbols().find_map(|symbol| match symbol {
            TopLevelStatement::FuncDefinition(f) if f.declaration.name == "main" => Some(f.clone()),
            _ => None,
        })
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &TopLevelStatement> {
        self.symbols.iter()
    }
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

#[derive(Debug, Clone)]
pub struct ZeaImportStatement {
    path: String,
}
#[derive(Debug, Clone)]
pub enum ZeaExportStatement {
    Func(String),
    GlobalConst(String),
}

mod utils {
    use crate::ast::{
        FuncDeclaration, FuncDefinition, Literal, StatementBlock, TopLevelStatement, ZeaExpression,
        ZeaModule, ZeaStatement, ZeaTypeIdent,
    };

    pub fn basic_module(
        name: impl Into<String>,
        symbols: impl Iterator<Item = TopLevelStatement>,
    ) -> ZeaModule {
        ZeaModule {
            path: name.into().into(),
            imports: vec![],
            exports: vec![],
            symbols: symbols.collect(),
        }
    }

    pub fn basic_main_returning(value: u8) -> FuncDefinition {
        FuncDefinition {
            declaration: FuncDeclaration {
                name: "main".to_string(),
                args: vec![],
                returns: ZeaTypeIdent::Basic("u8".to_string()),
            },
            body: StatementBlock(vec![return_literal_u8(value)]),
        }
    }

    pub fn return_literal_u8(value: u8) -> ZeaStatement {
        ZeaStatement::ReturnValue(literal_u8(value))
    }

    pub fn literal_u8(value: u8) -> ZeaExpression {
        ZeaExpression::Literal(Literal::Integer(value as u64))
    }
}
