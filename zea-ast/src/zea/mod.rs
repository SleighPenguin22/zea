#![allow(dead_code, unused_imports)]

pub mod datatype;
pub mod expression;
pub mod lowering;
pub mod patterns;
pub mod statement;
pub mod nodeexpansion;

pub use datatype::{Type, TypedIdentifier};
pub use expression::{Expression, ExpressionKind};
pub use statement::{Initialisation, Statement, StatementBlock, StatementKind, FunctionCall};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use zea_macros::HashEqById;

#[derive(Default, HashEqById)]
pub struct Module {
    pub id: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub globs: HashSet<Initialisation>,
    pub functions: HashSet<Function>,
}

impl Module {
    pub fn find_entry_point(&self) -> Option<&Function> {
        self.iter_symbols().find(|func| func.name == "main")
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter()
    }
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone, HashEqById)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: StatementBlock,
}
