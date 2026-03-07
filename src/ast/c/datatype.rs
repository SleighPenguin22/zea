use crate::ast::c::TypedIdentifier;
use std::collections::HashSet;

pub struct CStructDeclaration(String);
pub struct CStructDefinition {
    pub name: String,
    pub members: Vec<TypedIdentifier>,
}
pub struct CEnumDeclaration(String);
pub struct CUnionDeclaration(String);

pub struct CFunctionDeclaration(String, Vec<TypedIdentifier>);
#[derive(Debug, PartialEq, Clone)]
pub enum TypeSpecifier {
    Basic(String),
    Pointer(Box<TypeSpecifier>),
}
#[derive(Debug, PartialEq, Eq, Hash)]
#[derive(Clone)]
pub enum TypeQualifier {
    Static,
    Inline,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Type(HashSet<TypeQualifier>, TypeSpecifier);
