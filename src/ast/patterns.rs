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
    UnionVariant(String, String, Box<ZeaPattern>),
}

impl std::fmt::Display for ZeaPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            ZeaPattern::Ident(s) => s.clone(),
            ZeaPattern::ListHeadTail(head, tail) => {
                format!("[{head}, {tail}..]")
            }
            ZeaPattern::TupleUnpack(tups) => {
                let s: Vec<String> = tups.iter().map(|pat| pat.to_string()).collect();
                format!("({})", s.join(", "))
            }
            ZeaPattern::OptionSome(p) => format!("Some({p})"),
            ZeaPattern::OptionNone => "None".to_string(),
            ZeaPattern::UnionVariant(union, variant, pat) => {
                format!("{union}:{variant}({pat})")
            }
        };
        write!(f, "{}", s)
    }
}
