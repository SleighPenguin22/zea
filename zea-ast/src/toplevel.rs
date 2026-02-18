use crate::datatype::{TypedIdentifier, ZeaType};

pub struct FuncDeclaration {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: ZeaType,
}

pub struct FuncDefinition {}
