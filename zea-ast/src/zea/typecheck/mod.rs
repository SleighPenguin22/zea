use indexmap::IndexSet;

use crate::zea;

const BUILTIN_SCALAR_TYPES: [zea::TypeSpecifier; 9] = [
    zea::TypeSpecifier::t_Bool(),
    zea::TypeSpecifier::t_I8(),
    zea::TypeSpecifier::t_I16(),
    zea::TypeSpecifier::t_I32(),
    zea::TypeSpecifier::t_I64(),
    zea::TypeSpecifier::t_U8(),
    zea::TypeSpecifier::t_U16(),
    zea::TypeSpecifier::t_U32(),
    zea::TypeSpecifier::t_U64(),
    // TypeSpecifier::t_F32(),
    // TypeSpecifier::t_F64(),
    // TypeSpecifier::t_Unit(),
    // TypeSpecifier::t_Never(),
];
/// The id that a concrete type gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeConcreteId {
    id: usize,
}
impl std::fmt::Debug for TypeConcreteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeConcreteId({})", self.id)
    }
}

/// The id that a type-variable gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeVarId {
    id: usize,
}

impl std::fmt::Debug for TypeVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeVar({})", self.id)
    }
}
/// A table holding all unique types within a module.
#[derive(Debug)]
pub struct TypeInterningTable {
    type_ids: IndexSet<zea::TypeSpecifier>,
}
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum InferenceId {
    TypeConcrete(TypeConcreteId), // map some type-id to an actual type within an interning-table
    TypeVar(TypeVarId),
}
impl std::fmt::Debug for InferenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeConcrete(c) => write!(f, "InferenceId({:?})", c),
            Self::TypeVar(v) => write!(f, "InferenceId({:?})", v),
        }
    }
}

impl InferenceId {
    pub fn is_concrete(&self) -> bool {
        matches!(self, InferenceId::TypeConcrete(_))
    }
}

#[derive(Debug, Clone)]
pub enum TypeCheckError {}

pub struct ZeaTypeChecker {}
impl ZeaTypeChecker {
    pub fn new() -> Self {
        Self {}
    }
    pub fn check_module(&mut self, _module: &mut zea::Module) -> Result<(), TypeCheckError> {
        Ok(())
    }
}

impl Default for ZeaTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
