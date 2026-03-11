/// the left hand side of an assignment
///
/// The simplest is a basic identifier
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum AssignmentPattern {
    /// the pattern
    ///
    /// `var a: ...`
    ///
    /// or
    ///
    /// `a := ...`
    Identifier(String),
    /// the pattern
    ///
    /// `(<pat>, <pat>, <pat>) := ...`
    ///
    /// or
    ///
    /// `var (a,b,c) := ...`
    Tuple(Vec<AssignmentPattern>),
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum MatchPattern {
    /// the pattern `a => ...`
    Identifier(String),
    /// the pattern `(<pat>, <pat>, ...) => ...`
    Tuple(Vec<AssignmentPattern>),

    UnionVariant(String, String, Box<AssignmentPattern>),
}

impl std::fmt::Display for AssignmentPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            AssignmentPattern::Identifier(s) => s.clone(),
            AssignmentPattern::Tuple(tups) => {
                let s: Vec<String> = tups.iter().map(|pat| pat.to_string()).collect();
                format!("({})", s.join(", "))
            }
        };
        write!(f, "{}", s)
    }
}
