use std::fmt::{Debug, Formatter};

/// The Zea named Struct type / product type

pub struct StructDefinition {
    name: String,
    members: Vec<TypedIdentifier>,
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
    Ptr(Box<Type>),
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
            Type::Ptr(ptr) => &format!("&{ptr:?}"),
            // Type::Slice(slice) => &format!("&[{slice:?}]"),
        };

        write!(f, "{}", str)
    }
}

impl Into<Type> for &str {
    fn into(self) -> Type {
        Type::Basic(self.into())
    }
}

impl Into<Type> for String {
    fn into(self) -> Type {
        Type::Basic(self)
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TypedIdentifier(String, Type);
