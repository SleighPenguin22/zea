#![allow(dead_code, unused_imports)]
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;

#[cfg(test)]
pub mod test_utils;

pub use crate::ast::{
    datatype::{StructDefinition, Type, TypedIdentifier},
    expression::{Expression, Literal},
    patterns::AssignmentPattern,
    statement::{ConstInitialisation, Statement, StatementBlock},
};

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Debug, Default, Clone)]
pub struct Module {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub symbols: HashSet<TopLevelStatement>,
}
impl Module {
    pub fn find_entry_point(&self) -> Option<Function> {
        self.iter_symbols().find_map(|symbol| match symbol {
            TopLevelStatement::FuncDefinition(f) if f.name == "main" => Some(f.clone()),
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
    GlobalConst(ConstInitialisation),
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone)]
pub struct Function {
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

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.args.hash(state);
        self.returns.hash(state);
    }
}
