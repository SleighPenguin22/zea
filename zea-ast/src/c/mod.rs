#![allow(unused)]

pub mod datatype;
pub mod directives;
pub mod expression;
pub mod statement;

pub use crate::c::datatype::{Type, TypeQualifier, TypeSpecifier};
pub use crate::c::expression::Expression;
pub use crate::c::statement::{
    DerefReassignment, Initialisation, Reassignment, Statement, StatementBlock, VariableDeclaration,
};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub struct TypedIdentifier(pub Type, pub String);

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
    pub returns: Type,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
}

pub struct FunctionDefinition {
    pub declaration: FunctionDeclaration,
    pub body: StatementBlock,
}

pub enum TopLevelDecl {
    FuncDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
}

pub enum TopLevelDef {
    FuncDef(FunctionDefinition),
    VarInit(Initialisation),
}

pub struct TranslationUnit {
    directives: Vec<PreProcessorDirective>,
    declarations: Vec<TopLevelDecl>,
    symbols: Vec<TopLevelDef>,
}
