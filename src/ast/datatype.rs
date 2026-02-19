use std::fmt::{Debug, Formatter};

/// The Zea named Struct type / product type

pub struct ZeaStructDefinition {
    name: String,
    members: ZeaStructInner,
}

#[derive(PartialEq, PartialOrd, Eq)]
pub struct ZeaNamedStruct {
    name: String,
    members: ZeaStructInner,
}

pub type ZeaStructInner = Vec<TypedIdentifier>;

pub struct ZeaUnion {
    pub name: String,
    pub members: Vec<ZeaUnionVariant>,
}

pub enum ZeaUnionVariant {
    Tag(String),
    Type(TypedIdentifier),
}

/// The Type that is bundled with a:
/// - function parameter
/// - identifier in declaration(-assignments)
#[derive(PartialEq, PartialOrd, Eq, Clone, Hash)]
pub enum ZeaTypeIdent {
    /// Int, Bool, etc.
    Basic(String),
    /// `<type>&`
    Ptr(Box<ZeaTypeIdent>),
    /// `[<type>]`
    ArrayOf(Box<ZeaTypeIdent>),
    /// `[<type>]&`
    Slice(Box<ZeaTypeIdent>),
    /// `<type>?`
    Option(Box<ZeaTypeIdent>),
}

impl Debug for ZeaTypeIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ZeaTypeIdent::Basic(typ) => typ,
            ZeaTypeIdent::ArrayOf(arr) => &format!("[{arr:?}]"),
            ZeaTypeIdent::Option(opt) => &format!("{opt:?}?"),
            ZeaTypeIdent::Ptr(ptr) => &format!("{ptr:?}&"),
            ZeaTypeIdent::Slice(slice) => &format!("[{slice:?}]&"),
        };

        write!(f, "{}", str)
    }
}

impl Into<ZeaTypeIdent> for &str {
    fn into(self) -> ZeaTypeIdent {
        ZeaTypeIdent::Basic(self.into())
    }
}

impl Into<ZeaTypeIdent> for String {
    fn into(self) -> ZeaTypeIdent {
        ZeaTypeIdent::Basic(self)
    }
}

impl ZeaTypeIdent {
    pub fn ptr(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Ptr(Box::new(typ))
    }
    pub fn array(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::ArrayOf(Box::new(typ))
    }

    pub fn slice(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Slice(Box::new(typ))
    }

    pub fn option(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Option(Box::new(typ))
    }
}

pub type TypedIdentifier = (String, ZeaTypeIdent);
