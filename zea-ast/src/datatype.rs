#[derive(PartialEq, PartialOrd, Eq, Debug)]
pub enum ZeaType {
    /// Int, Bool, etc.
    Basic(String),
    /// `<type>&`
    Ptr(Box<ZeaType>),
    /// `[<type>]`
    ArrayOf(Box<ZeaType>),
    /// `[<type>]&`
    Slice(Box<ZeaType>),
    /// `<type>?`
    Option(Box<ZeaType>),
}

impl Into<ZeaType> for &str {
    fn into(self) -> ZeaType {
        ZeaType::Basic(self.into())
    }
}

impl Into<ZeaType> for String {
    fn into(self) -> ZeaType {
        ZeaType::Basic(self)
    }
}

impl ZeaType {
    pub fn ptr(typ: ZeaType) -> ZeaType {
        ZeaType::Ptr(Box::new(typ))
    }
    pub fn array(typ: ZeaType) -> ZeaType {
        ZeaType::ArrayOf(Box::new(typ))
    }

    pub fn slice(typ: ZeaType) -> ZeaType {
        ZeaType::Slice(Box::new(typ))
    }

    pub fn option(typ: ZeaType) -> ZeaType {
        ZeaType::Option(Box::new(typ))
    }
}

pub type TypedIdentifier = (String, ZeaType);
