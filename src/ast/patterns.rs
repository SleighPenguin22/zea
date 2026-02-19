/// Lhs's of decl-assignments, reassignments and match-arms
///
/// The simplest is a basic identifier
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum ZeaPattern {
    /// the pattern `a => ...`
    /// or
    /// `const a := ...`
    Ident(String),
    /// the pattern `(<pat>, <pat>, <pat>) => ...`
    ///
    /// or
    ///
    /// `const (a,b,c) = ...`
    TupleUnpack(Vec<ZeaPattern>),
    /// the pattern `[a,b...] => ...`
    ///
    /// or
    ///
    /// `const [head,rest..] = split-head(...)`
    ListHeadTail(String, String),
    /// the pattern `std:option:Some(<pat>) => ...`
    OptionSome(Box<ZeaPattern>),
    /// the pattern `std:option:None => ...`
    OptionNone,
    /// the pattern `union:Variant(<pat>) => ...`
    UnionVariant(String, Box<ZeaPattern>),
}
