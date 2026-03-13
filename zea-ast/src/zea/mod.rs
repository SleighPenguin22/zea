#![allow(dead_code, unused_imports)]
mod nodeexpansion;
use std::hash::{Hash, Hasher};
use zea_macros::HashEqById;

#[derive(Default, HashEqById)]
pub struct Module {
    pub id: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub globs: Vec<Initialisation>,
    pub functions: Vec<Function>,
}

impl Module {
    pub fn find_entry_point(&self) -> Option<&Function> {
        self.iter_symbols().find(|func| func.name == "main")
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter()
    }
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone, HashEqById)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: StatementBlock,
}

#[derive(Debug, Clone, HashEqById)]
pub struct Statement {
    pub id: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    // initial pass
    /// Variable initialisation
    Initialisation(Initialisation),
    /// Variable Reassignment
    Reassignment(Reassignment),
    FunctionCall(FunctionCall),
    /// Control-flow return
    Return(Expression),
    /// A tailing expression in a block
    BlockTail(Expression),

    /// A Block of statements
    Block(StatementBlock),
    // CondMatch(Box<ConditionMatch>),

    // after expansion
    ExpandedBlock(ExpandedBlockExpr),
    ExpandedInitialisation(ExpandedInitialisation),
    SimpleInitialisation(SimpleInitialisation),
}

#[derive(Debug, Clone, HashEqById)]
pub struct Initialisation {
    pub id: usize,
    pub typ: Option<Type>,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}
#[derive(Debug, Clone, HashEqById)]
pub struct Reassignment {
    pub id: usize,
    pub assignee: String,
    pub value: Expression,
}

#[derive(Debug, Clone, HashEqById)]
pub struct FunctionCall {
    pub id: usize,
    pub name: String,
    pub args: Vec<Expression>,
}
#[derive(Debug, Clone, HashEqById)]
pub struct StatementBlock {
    pub id: usize,
    pub statements: Vec<Statement>,
}

impl IntoIterator for StatementBlock {
    type Item = Statement;
    type IntoIter = <Vec<Statement> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.statements.into_iter()
    }
}
impl StatementBlock {
    pub fn as_slice(&self) -> &[Statement] {
        self.statements.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedBlockExpr {
    /// The label that the block expression has its value assigned to
    /// i.e. `__block0`, `__block1` etc.
    /// This label must be unique to the scope of the function in which it exists
    pub id: usize,
    pub statements: Vec<Statement>,
    pub last: Expression,
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, Clone, HashEqById)]
pub struct SimpleInitialisation {
    pub id: usize,
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: Expression,
}

#[derive(Debug, Clone, HashEqById)]
pub struct ExpandedInitialisation {
    pub id: usize,
    pub temporary: SimpleInitialisation,
    pub unpacked_assignments: Vec<ExpandedInitialisation>,
}

// macro_rules! extended {
//     ($($first:expr),+) => {{
//         vec![$($first),+]
//     }};
//     ($($first:expr),+ ; $($rest:expr),+) => {{
//         let mut v = vec![$($first),+];
//         $(v.extend($rest);)+
//         v
//     }};
//
//     (; $($rest:expr),+) => {{
//         let mut v = Vec::new();
//         $(v.extend($rest);)+
//         v
//     }};
// }

#[derive(Debug, Clone, HashEqById)]
pub struct Expression {
    pub id: usize,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    // initial pass
    Unit,
    IntegerLiteral(u64),
    BoolLiteral(bool),
    FloatLiteral(f64),
    StringLiteral(String),
    Ident(String),
    FuncCall(FunctionCall),
    BinOpExpr(BinOp, Box<Expression>, Box<Expression>),
    UnOpExpr(UnOp, Box<Expression>),

    Block(StatementBlock),
    // PatternMatch(PatternMatch),
    // ConditionMatch(ConditionMatch),
    // IfThenElse(IfThenElse),

    // after expansion
    ExpandedBlock(Box<ExpandedBlockExpr>),
}

impl Expression {
    pub fn unit(id: usize) -> Self {
        Expression {
            id,
            kind: ExpressionKind::Unit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogAnd,
    LogOr,
    LogXor,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}

#[derive(Clone, Debug, HashEqById)]
pub struct ConditionMatch {
    pub id: usize,
    conditions: Vec<ConditionMatchArm>,
}

#[derive(Clone, Debug, HashEqById)]
pub struct PatternMatch {
    pub id: usize,
    subject: Box<Expression>,
    patterns: Vec<PatternMatchArm>,
}

#[derive(Clone, Debug, HashEqById)]
pub struct IfThenElse {
    pub id: usize,
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Option<Box<Expression>>,
}

#[derive(Clone, Debug, HashEqById)]
pub struct ExpandedIfThenElse {
    pub id: usize,
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Box<Expression>,
}
#[derive(Clone, Debug, HashEqById)]
pub struct PatternMatchArm {
    pub id: usize,
    pat: AssignmentPattern,
    value: Box<Expression>,
}
#[derive(Clone, Debug, HashEqById)]
pub struct ConditionMatchArm {
    pub id: usize,
    case: Box<Expression>,
    value: Box<Expression>,
}
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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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
    pub fn new(typ: Type, ident: impl Into<String>) -> Self {
        Self(ident.into(), typ)
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
