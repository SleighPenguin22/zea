use crate::c::TypedIdentifier;
use std::collections::HashSet;

pub struct CStructDeclaration(String);
pub struct CStructDefinition {
    pub name: String,
    pub members: Vec<TypedIdentifier>,
}
pub struct CEnumDeclaration(String);
pub struct CUnionDeclaration(String);

pub struct CFunctionDeclaration(pub TypedIdentifier, pub Vec<TypedIdentifier>);
#[derive(Debug, PartialEq, Clone)]
pub enum TypeSpecifier {
    Basic(String),
    Pointer(Box<TypeSpecifier>),
}

pub type DerefAssignee = TypeSpecifier;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TypeQualifier {
    Static,
    Inline,
}

impl From<TypeQualifier> for String {
    fn from(value: TypeQualifier) -> Self {
        match value {
            TypeQualifier::Static => "static".to_string(),
            TypeQualifier::Inline => "inline".to_string(),
        }
    }
}

impl From<&TypeQualifier> for String {
    fn from(value: &TypeQualifier) -> Self {
        match value {
            TypeQualifier::Static => "static".to_string(),
            TypeQualifier::Inline => "inline".to_string(),
        }
    }
}

impl TypeSpecifier {
    /// Get the type behind any pointer variant:
    /// ```ignore
    /// "int" => "int",
    /// Pointer("int") => "int",
    /// Pointer(Pointer("bool")) => "bool"
    /// ```
    pub fn get_deepest(&self) -> String {
        match self {
            TypeSpecifier::Basic(t) => t.clone(),
            TypeSpecifier::Pointer(t) => t.as_ref().get_deepest(),
        }
    }
}

impl From<TypeSpecifier> for Type {
    fn from(value: TypeSpecifier) -> Self {
        Self {
            qualifiers: HashSet::new(),
            specifier: value,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub qualifiers: HashSet<TypeQualifier>,
    pub specifier: TypeSpecifier,
}
