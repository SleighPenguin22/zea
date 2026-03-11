use std::fmt::{Debug, Formatter};

/// The Zea named Struct type / product type
pub struct StructDefinition {
    name: String,
    members: Vec<TypedIdentifier>,
}

pub struct TupleSignature {
    members: Vec<Type>,
}

pub struct Union {
    pub name: String,
    pub members: Vec<UnionVariant>,
}

pub enum UnionVariant {
    Tag(String),
    Type(TypedIdentifier),
}

/// The Type that is bundled with a:
/// - function parameter
/// - identifier in declaration(-assignments)
#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Type {
    /// Int, Bool, etc.
    Basic(String),

    /// `<type>&`
    Pointer(Box<Type>),
    /// `[<type>]`
    ArrayOf(Box<Type>),
    // /// `&[<type>]`
    // Slice(Box<Type>),
    // /// `?<type>`
    // Option(Box<Type>),
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Type::Basic(typ) => typ,
            Type::ArrayOf(arr) => &format!("[{arr:?}]"),
            // Type::Option(opt) => &format!("?{opt:?}"),
            Type::Pointer(ptr) => &format!("&{ptr:?}"),
            // Type::Slice(slice) => &format!("&[{slice:?}]"),
        };

        write!(f, "{}", str)
    }
}

impl From<&str> for Type {
    fn from(val: &str) -> Self {
        Type::Basic(val.into())
    }
}

impl From<String> for Type {
    fn from(val: String) -> Self {
        Type::Basic(val)
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TypedIdentifier(String, Type);
impl TypedIdentifier {
    pub fn new(typ: Type, ident: String) -> Self {
        Self(ident, typ)
    }
}

impl TypedIdentifier {
    pub fn ident(&self) -> &str {
        &self.0
    }
    pub fn typ(&self) -> &Type {
        &self.1
    }
}
