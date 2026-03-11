#![allow(dead_code, unused_imports)]

pub mod datatype;
pub mod expression;
pub mod lowering;
pub mod patterns;
pub mod statement;
#[cfg(test)]
pub mod test_utils;

pub use datatype::{Type, TypedIdentifier};
pub use statement::{Initialisation, StatementBlock};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use zea_macros::HashById;

#[derive(Default, HashById)]
pub struct Module {
    pub id: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub symbols: HashSet<TopLevelStatement>,
}

impl Module {
    pub fn find_entry_point(&self) -> Option<&Function> {
        self.iter_symbols().find_map(|symbol| match symbol {
            TopLevelStatement::FuncDefinition(f) if f.name == "main" => Some(f),
            _ => None,
        })
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &TopLevelStatement> {
        self.symbols.iter()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum TopLevelStatement {
    FuncDefinition(Function),
    GlobalConst(Initialisation),
}

impl Eq for TopLevelStatement {}
impl Hash for TopLevelStatement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TopLevelStatement::GlobalConst(init) => init.assignee.hash(state),
            TopLevelStatement::FuncDefinition(func) => {
                func.name.hash(state);
                func.returns.hash(state);
            }
        }
    }
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone, HashById)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: StatementBlock,
}
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Function {}

