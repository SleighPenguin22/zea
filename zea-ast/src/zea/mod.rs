#![allow(dead_code, unused_imports)]
mod nodeexpansion;
pub use crate::zea::nodeexpansion::NodeExpander;
use crate::PrettyAST;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use zea_macros::{HashEqById, VariantToStr};

#[derive(Default, HashEqById, Debug)]
pub struct Module {
    pub id: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub globs: Vec<Initialisation>,
    pub functions: Vec<Function>,
}
macro_rules! indent {
    ($d:expr) => {{
        let d: usize = $d;
        " ".repeat((d + 1) * 4)
    }};
}
impl PrettyAST for Module {
    fn pretty_print(&self, depth: usize) -> String {
        let mut buffer = "Module(\n".pretty_print(depth);
        let imports = if self.imports.is_empty() {
            "#IMPORTS NOTHING\n".pretty_print(depth + 1)
        } else {
            let mut imp_buffer = "#IMPORTS".pretty_print(depth + 1);
            for e in self.imports.iter() {
                imp_buffer += &e.pretty_print(depth + 2);
            }
            imp_buffer
        };
        buffer += &imports;

        let exports = if self.exports.is_empty() {
            "#EXPORTS NOTHING\n".pretty_print(depth + 1)
        } else {
            let mut exp_buffer = "#EXPORTS".pretty_print(depth + 1);
            for e in self.exports.iter() {
                exp_buffer += &e.pretty_print(depth + 1);
            }
            exp_buffer
        };
        buffer += &exports;

        buffer += &"#GLOBS".pretty_print(depth + 1);
        for glob in self.globs.iter() {
            buffer += &glob.pretty_print(depth + 2);
        }
        buffer += &"/#GLOBS".pretty_print(depth + 1);

        buffer += &"#FUNCS".pretty_print(depth + 1);
        for func in self.functions.iter() {
            buffer += &func.pretty_print(depth + 2);
        }
        buffer += &"/#FUNCS\n".pretty_print(depth + 1);

        buffer += &"/MODULE".pretty_print(depth);
        buffer
    }
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
pub struct HoistedFunctionSignature {
    pub id: usize,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
}

impl From<Function> for HoistedFunctionSignature {
    fn from(value: Function) -> Self {
        HoistedFunctionSignature {
            id: value.id,
            name: value.name,
            args: value.args,
            returns: value.returns,
        }
    }
}

impl PrettyAST for Function {
    fn pretty_print(&self, depth: usize) -> String {
        format!(
            "{0}{2}{3:?} -> {4:?}\n\
            {1}BODY\n\
            {5}\
            {1}/BODY\n\
            {0}/FUNC {2}\n",
            indent!(depth),
            indent!(depth + 1),
            self.name,
            self.args,
            self.returns,
            self.body.pretty_print(depth + 2)
        )
    }
}

impl PrettyAST for StatementBlock {
    fn pretty_print(&self, depth: usize) -> String {
        let mut buffer = indent!(depth) + "BLOCK\n";

        for s in self.statements.iter() {
            buffer += &s.pretty_print(depth + 1);
        }
        buffer += &"/BLOCK\n".pretty_print(depth);

        buffer
    }
}

impl PrettyAST for ExpandedBlockExpr {
    fn pretty_print(&self, depth: usize) -> String {
        let mut buffer = "BLOCK\n".pretty_print(depth);

        for s in self.statements.iter() {
            buffer += &format!("{}STMT\n{}", indent!(depth), s.pretty_print(depth + 1));
        }
        buffer += &self.last.pretty_print(depth + 1);

        buffer += &"/BLOCK\n".pretty_print(depth);

        buffer
    }
}

impl PrettyAST for Statement {
    fn pretty_print(&self, depth: usize) -> String {
        match &self.kind {
            StatementKind::Return(e) => {
                format!("{}RETURN\n{}", indent!(depth), e.pretty_print(depth + 1))
            }
            StatementKind::Initialisation(i) => i.pretty_print(depth),
            StatementKind::BlockTail(e) => {
                format!("{}TAIL\n{}", indent!(depth), e.pretty_print(depth + 1))
            }
            _ => todo!("pretty print statement with kind {:?}", self.kind),
        }
    }
}
impl PrettyAST for Initialisation {
    fn pretty_print(&self, depth: usize) -> String {
        match &self.kind {
            InitialisationKind::Packed(p) => p.pretty_print(depth),
            InitialisationKind::PartiallyUnpacked(p) => p.pretty_print(depth),
        }
    }
}

impl PrettyAST for PackedInitialisation {
    fn pretty_print(&self, depth: usize) -> String {
        format!(
            "{0}P_INIT\n\
        {1}#PATTERN:\n\
        {2}\n\
        {1}#TYPE\n\
        {3}\n\
        {1}#VALUE:\n\
        {4}\n\
        {0}/P_INIT\n",
            indent!(depth),
            indent!(depth + 1),
            self.assignee.pretty_print(depth + 2),
            self.typ.pretty_print(depth + 2),
            self.value.pretty_print(depth + 2)
        )
    }
}

impl PrettyAST for String {
    fn pretty_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}

impl<'a> PrettyAST for &'a str {
    fn pretty_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}

impl PrettyAST for Type {
    fn pretty_print(&self, depth: usize) -> String {
        format!("{:?}\n", self).pretty_print(depth)
    }
}

impl PrettyAST for Option<Type> {
    fn pretty_print(&self, depth: usize) -> String {
        match self {
            None => String::from("TO BE INFERRED").pretty_print(depth),
            Some(t) => t.pretty_print(depth),
        }
    }
}

impl PrettyAST for UnpackedInitialisation {
    fn pretty_print(&self, depth: usize) -> String {
        format!(
            "{0}UNP_INIT(\n\
        {1}#PATTERN:\n\
        {2}\
        {1}#TYPE\n\
        {3}\
        {1}#VALUE:\n\
        {4}\
        {0}/UNP_INIT\n",
            indent!(depth),
            indent!(depth + 1),
            self.assignee.pretty_print(depth + 1),
            self.typ.pretty_print(depth + 2),
            self.value.pretty_print(depth + 2)
        )
    }
}

impl PrettyAST for PartiallyUnpackedInitialisation {
    fn pretty_print(&self, depth: usize) -> String {
        let mut buffer = self.temporary.pretty_print(depth);
        for u in self.unpacked_assignments.iter() {
            buffer += &u.pretty_print(depth + 1);
        }
        buffer
    }
}

impl PrettyAST for AssignmentPattern {
    fn pretty_print(&self, depth: usize) -> String {
        match self {
            AssignmentPattern::Identifier(i) => indent!(depth) + i,
            AssignmentPattern::Tuple(tup) => {
                indent!(depth)
                    + &tup
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
            }
        }
    }
}
#[derive(Debug, Clone, HashEqById)]
pub struct Statement {
    pub id: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq, VariantToStr)]
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
}

impl PrettyAST for Expression {
    fn pretty_print(&self, depth: usize) -> String {
        let kind_str = self.kind.variant_as_str();
        match &self.kind {
            ExpressionKind::Ident(i) => format!("{}{kind_str}({i})\n", indent!(depth)),
            ExpressionKind::IntegerLiteral(i) => format!("{}Int({i})\n", indent!(depth)),
            ExpressionKind::FloatLiteral(i) => format!("{}Float({i})\n", indent!(depth)),
            ExpressionKind::BinOpExpr(op, l, r) => {
                format!(
                    "{0}{op:?}\n{1}{2}{0}/`{op:?}`\n",
                    indent!(depth),
                    l.pretty_print(depth + 1),
                    r.pretty_print(depth + 1),
                )
            }
            ExpressionKind::UnOpExpr(op, arg) => {
                format!(
                    "{0}{op:?}\n{1}{0}/`{op:?}`\n",
                    indent!(depth),
                    arg.pretty_print(depth + 1),
                )
            }
            ExpressionKind::MemberAccess(e, m) => {
                format!(
                    "{0}MEMBER\n{1}{2}",
                    indent!(depth),
                    e.pretty_print(depth + 1),
                    m.pretty_print(depth + 1),
                )
            }
            _ => todo!("pretty print expression of kind {:?}", self.kind),
        }
    }
}

#[derive(Debug, Clone, HashEqById)]
pub struct Initialisation {
    pub id: usize,
    pub kind: InitialisationKind,
}

impl Initialisation {
    pub fn packed(
        id: usize,
        typ: Option<Type>,
        assignee: AssignmentPattern,
        value: Expression,
    ) -> Self {
        Self {
            id,
            kind: InitialisationKind::Packed(PackedInitialisation {
                typ,
                assignee,
                value,
            }),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PackedInitialisation {
    pub typ: Option<Type>,
    pub assignee: AssignmentPattern,
    pub value: Expression,
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnpackedInitialisation {
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: Expression,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PartiallyUnpackedInitialisation {
    pub temporary: UnpackedInitialisation,
    pub unpacked_assignments: Vec<Initialisation>,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InitialisationKind {
    Packed(PackedInitialisation),
    PartiallyUnpacked(PartiallyUnpackedInitialisation),
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

impl Expression {
    pub fn label_member_access(
        generator: &mut NodeExpander,
        e: Expression,
        field: usize,
    ) -> Expression {
        Expression {
            id: generator.label(),
            kind: ExpressionKind::MemberAccess(Box::new(e), format!("_{field}")),
        }
    }

    pub fn ident(generator: &mut NodeExpander, ident: String) -> Expression {
        Expression {
            id: generator.label(),
            kind: ExpressionKind::Ident(ident),
        }
    }
}

#[derive(Debug, Clone, PartialEq, VariantToStr)]
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
    MemberAccess(Box<Expression>, String),

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
    Subscript,
    Lsh,
    Rsh,
    Eq,
    Neq,
    Geq,
    Leq,
    LT,
    GT,
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
pub struct TypedIdentifier {
    pub name: String,
    pub typ: Type,
}

impl TypedIdentifier {
    pub fn new(typ: Type, name: impl Into<String>) -> Self {
        Self {
            typ,
            name: name.into(),
        }
    }
}
