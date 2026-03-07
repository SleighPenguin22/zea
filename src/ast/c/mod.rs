#![allow(unused)]

pub mod datatype;
pub mod directives;
pub mod expression;
pub mod statement;

use crate::ast::c::datatype::Type;
use crate::ast::c::expression::Expression;
use crate::ast::c::statement::{Initialisation, StatementBlock, VariableDeclaration};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub struct TypedIdentifier(Type, String);

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
}

pub enum PreProcessorDirective {
    PragmaOnce,
    Include(String),
    Define(String, String),
    // __FILE__,
}

pub struct FunctionDeclaration {
    returns: Type,
    name: String,
    args: Vec<TypedIdentifier>,
}

pub struct FunctionDefinition {
    declaration: FunctionDeclaration,
    body: StatementBlock,
}

pub enum TopLevelItem {
    FuncDecl(FunctionDeclaration),
    FuncDef(FunctionDefinition),
    VarDecl(VariableDeclaration),
    VarInit(Initialisation),
}

pub struct TranslationUnit {
    symbols: Vec<TopLevelItem>,
}
