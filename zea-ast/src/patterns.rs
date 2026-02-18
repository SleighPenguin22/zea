use crate::datatype::{TypedIdentifier, ZeaType};

/// Lhs's of decl-assignments, reassignments and match-arms
///
/// The simplest is a basic identifier
pub enum ZeaPattern {
    /// `a => ...`
    /// or
    /// `const a := ...`
    Ident(String),
    /// `(<pat>, <pat>, <pat>) => ...`
    /// or
    /// `const (a,b,c) = ...`
    TupleUnpack(Vec<ZeaPattern>),
    /// `[a,b..] => ...`
    /// or
    /// `const [head,rest..] = split-head(...)`
    ListHeadTail(String, String),
    /// `std:option:Some(<pat>) => ...`
    OptionSome(Box<ZeaPattern>),
    /// `std:option:None => ...`
    OptionNone,
    /// `union:Variant(<pat>) => ...`
    UnionVariant(String, Box<ZeaPattern>),
}
