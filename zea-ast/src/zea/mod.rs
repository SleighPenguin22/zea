#![allow(dead_code, unused_imports, unused_macro_rules, unused_macros)]
/// This module contains the AST definition for the Zea language.
/// Any node that encompasses some structure with meaningful data has an id, this id has the following guarantees:
/// - the id is unique
/// - there is no specified order in the id's of nodes.
/// - the ID of a node stays the same through the whole compilation process.
///
/// As such, you can use these id's as keys in hashtables tables that annotate nodes.
///
/// To maintain these invariants, any AST-visitors must not change the ID of an existing node.
/// When the visitor places a new node in the AST, that node must get a new unique ID.
///
/// To make this easier, the [`visitors::altering::NodeLabeler`] trait
/// may be implemented on a visitor.
/// This trait provides the [`visitors::altering::NodeLabeler::labeler_from`] method
/// to maintain the generation of unique ID's.
///
/// A node having an ID of 0 signals a sentinel ID,
/// it signifies that the node still requires a unique ID.
///
/// The [`visitors::altering::BareNodeLabeler`] visitor
/// grants a unique ID to node with a sentinel ID.
use crate::helper_impls::StructuralEq;
use crate::zea::hir_nodes::HIRASTNode;
use crate::zea::hir_nodes::HIRModule;
use crate::zea::visitors::Visitor;

mod typecheck;
pub use typecheck::typecheck_module;
mod impls;
mod mir_nodes;
pub mod visitors;
pub mod hir_nodes {
    use std::{fmt::Debug, fmt::Formatter, hash::Hash, hash::Hasher};

    use zea_internal_macros::{ASTStructuralEq, VariantToStr};

    use crate::{
        helper_impls::StructuralEq,
        zea::{BinOp, HIRScopedIdentifier, UnOp},
    };

    use super::NodeId;
    pub enum HIRASTNode {
        Module(HIRModule),
        Function(HIRFunction),
        Block(HIRBlockExpression),
        Branch(HIRBranch),
        Expression(HIRExpression),
    }

    macro_rules! variant_to_some {
        ($selfT:ident, $self:ident, $variantname:ident) => {
            match $self {
                $selfT::$variantname(var) => Some(var),
                _ => None,
            }
        };
    }
    impl HIRASTNode {
        pub fn as_module(self) -> Option<HIRModule> {
            variant_to_some!(Self, self, Module)
        }
        pub fn as_block(self) -> Option<HIRBlockExpression> {
            variant_to_some!(Self, self, Block)
        }
        pub fn as_function(self) -> Option<HIRFunction> {
            variant_to_some!(Self, self, Function)
        }
        pub fn as_branch(self) -> Option<HIRBranch> {
            variant_to_some!(Self, self, Branch)
        }
        pub fn as_expr(self) -> Option<HIRExpression> {
            variant_to_some!(Self, self, Expression)
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct HIRModule {
        pub id: NodeId,
        pub imports: Vec<String>,
        pub exports: Vec<String>,
        pub global_vars: Vec<HIRInitializationBlock>,
        pub functions: Vec<HIRFunction>,
        pub struct_definitions: Vec<HIRStructDataTypeDefinition>,
    }

    impl HIRModule {
        pub fn find_entry_point(&self) -> Option<&HIRFunction> {
            self.iter_functions().find(|func| func.name == "main")
        }

        pub fn iter_functions(&self) -> impl Iterator<Item = &HIRFunction> {
            self.functions.iter()
        }
        pub fn iter_global_vars(&self) -> impl Iterator<Item = &HIRInitializationBlock> {
            self.global_vars.iter()
        }
        pub fn iter_structs(&self) -> impl Iterator<Item = &HIRStructDataTypeDefinition> {
            self.struct_definitions.iter()
        }
    }

    #[derive(Debug, Clone)]
    pub struct HIRFuncParam {
        pub id: NodeId,
        pub typ: HIRTypeSpecifier,
        pub name: String,
    }

    /// A top-level function definition
    ///
    /// Function may be defined only once within a module.
    /// Functions may be imported as many times as needed.
    #[derive(Debug, Clone)]
    pub struct HIRFunction {
        pub id: NodeId,
        pub name: String,
        pub params: Vec<HIRFuncParam>,
        pub returns: HIRTypeSpecifier,
        pub body: HIRBlockExpression,
    }

    #[derive(Debug, Clone)]
    pub struct HoistedFunctionSignature {
        pub id: NodeId,
        pub name: String,
        pub args: Vec<HIRFuncParam>,
        pub returns: HIRTypeSpecifier,
    }

    #[derive(Debug, Clone)]
    pub struct HIRStatement {
        pub id: NodeId,
        pub kind: HIRStatementKind,
    }

    #[derive(Debug, Clone, VariantToStr)]
    pub enum HIRStatementKind {
        // initial pass
        /// Variable initialization
        Initialization(HIRInitializationBlock),
        /// Variable Reassignment
        Reassignment(HIRReassignment),
        FunctionCall(HIRFunctionCall),
        /// Control-flow return
        Return(HIRExpression),
        /// A tailing expression in a block
        BlockTail(HIRExpression),

        // CondMatch(Box<ConditionMatch>),

        // after expansion
        Block(HIRBlockExpression),
        IfThenElse(HIRBranch),
    }

    /// A packed or unpacked initialisation
    #[derive(Debug, Clone)]
    pub struct HIRInitializationBlock {
        pub id: NodeId,
        pub kind: HIRInitializationKind,
    }
    impl StructuralEq for HIRInitializationBlock {
        fn eq_ignore_id(&self, other: &Self) -> bool {
            let mut is_eq = true;
            is_eq &= (self.kind).eq_ignore_id(&other.kind);
            is_eq
        }
    }

    impl HIRInitializationBlock {
        pub fn packed(
            typ: Option<HIRTypeSpecifier>,
            assignee: HIRAssignmentPattern,
            value: HIRExpression,
        ) -> Self {
            Self {
                id: NodeId::sentinel(),
                kind: HIRInitializationKind::Packed(HIRPackedInitialization {
                    typ,
                    assignee,
                    value,
                }),
            }
        }
    }
    /// An assignment to a pattern
    /// This node is an intermediate, high level node that gets desugared into a series of assignments
    ///
    /// Has one of the forms
    /// - `var: type? = value`
    /// - `@(pat1, pat2, ..., patN): type? = value`
    ///
    /// NOTE:
    /// Support for enum destructuring to be added later
    #[derive(Debug, Clone)]
    pub struct HIRPackedInitialization {
        pub typ: Option<HIRTypeSpecifier>,
        pub assignee: HIRAssignmentPattern,
        pub value: HIRExpression,
    }

    impl HIRPackedInitialization {
        pub fn untyped(assignee: HIRAssignmentPattern, value: HIRExpression) -> Self {
            Self {
                typ: None,
                assignee,
                value,
            }
        }
    }

    /// An assignment to a simple, totally unpacked variable.
    ///
    /// Has one of the forms
    /// - `var := value`
    /// - `var: type = value`
    #[derive(Debug, Clone)]
    pub struct HIRSimpleInitialization {
        pub id: NodeId,
        pub typ: Option<HIRTypeSpecifier>,
        pub assignee: String,
        pub value: HIRExpression,
    }

    impl HIRSimpleInitialization {
        pub fn untyped(assignee: &str, value: HIRExpression) -> Self {
            Self {
                id: NodeId::sentinel(),
                assignee: assignee.to_string(),
                value,
                typ: None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct HIRPartiallyUnpackedInitialization {
        pub temporary: HIRSimpleInitialization,
        pub unpacked_assignments: Vec<HIRInitializationBlock>,
    }

    #[derive(Debug, Clone)]
    pub enum HIRInitializationKind {
        Packed(HIRPackedInitialization),
        Unpacked(Vec<HIRSimpleInitialization>),
    }

    #[derive(Debug, Clone)]
    pub struct HIRReassignment {
        pub id: NodeId,
        pub assignee: String,
        pub value: HIRExpression,
    }

    impl HIRReassignment {
        pub fn wrap_in_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::Reassignment(self),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct HIRFunctionCall {
        pub id: NodeId,
        pub subject: Box<HIRExpression>,
        pub args: Vec<HIRExpression>,
    }

    impl HIRFunctionCall {
        pub fn wrap_in_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::FunctionCall(self),
            }
        }

        pub fn wrap_in_expression(self) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::FunctionCall(self),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct HIRBlockExpression {
        /// The label that the block expression has its value assigned to
        /// i.e. `__block0`, `__block1` etc.
        /// This label must be unique to the scope of the function in which it exists
        pub id: NodeId,
        pub statements: Vec<HIRStatement>,
        pub last: HIRExpression,
    }

    impl HIRBlockExpression {
        pub fn wrap_in_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::Block(self),
            }
        }
        pub fn wrap_in_expression(self) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::Block(Box::new(self)),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct HIRExpression {
        pub id: NodeId,
        pub kind: HIRExpressionKind,
    }

    impl HIRExpression {
        pub fn with_id(mut self, id: NodeId) -> Self {
            self.id = id;
            self
        }
    }

    impl HIRExpression {
        pub fn tuple_member_access(e: HIRExpression, field: usize) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::MemberAccess(Box::new(e), format!("_{field}")),
            }
        }

        pub fn ident(ident: String) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::UnScopedIdent(ident),
            }
        }
        pub fn scoped_local(ident: String, origin: NodeId) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::ScopedIdent(HIRScopedIdentifier::local(origin, ident)),
            }
        }

        pub fn wrap_in_return_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::Return(self),
            }
        }

        pub fn wrap_in_block_tail_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::BlockTail(self),
            }
        }

        pub fn wrap_lit_int(i: usize) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::IntegerLiteral(i),
            }
        }
        pub fn wrap_lit_float(f: f64) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::FloatLiteral(f),
            }
        }

        pub fn wrap_lit_bool(b: bool) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::BoolLiteral(b),
            }
        }

        pub fn wrap_ident(ident: String) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::UnScopedIdent(ident),
            }
        }
    }

    #[derive(Debug, Clone, VariantToStr)]
    pub enum HIRExpressionKind {
        // initial pass
        Unit,
        IntegerLiteral(usize),
        BoolLiteral(bool),
        FloatLiteral(f64),
        StringLiteral(String),
        UnScopedIdent(String),
        ScopedIdent(HIRScopedIdentifier),
        FunctionCall(HIRFunctionCall),
        BinOpExpr(BinOp, Box<HIRExpression>, Box<HIRExpression>),
        UnOpExpr(UnOp, Box<HIRExpression>),
        MemberAccess(Box<HIRExpression>, String),
        IfThenElse(HIRBranch),

        // PatternMatch(PatternMatch),
        // ConditionMatch(ConditionMatch),
        // IfThenElse(IfThenElse),

        // after expansion
        Block(Box<HIRBlockExpression>),
    }

    impl HIRExpression {
        pub const fn unit() -> Self {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::Unit,
            }
        }

        pub fn binop(op: BinOp, l: HIRExpression, r: HIRExpression) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::BinOpExpr(op, Box::new(l), Box::new(r)),
            }
        }
        pub fn unop(op: UnOp, e: HIRExpression) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::UnOpExpr(op, Box::new(e)),
            }
        }

        pub fn member_access(data: HIRExpression, member: String) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::MemberAccess(Box::new(data), member),
            }
        }
    }

    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct ConditionMatch {
        pub id: NodeId,
        conditions: Vec<HIRConditionMatchArm>,
    }

    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct PatternMatch {
        pub id: NodeId,
        patterns: Vec<HIRPatternMatchArm>,
        subject: Box<HIRExpression>,
    }

    #[derive(Clone, Debug)]
    pub struct HIRBranch {
        pub id: NodeId,
        pub condition: Box<HIRExpression>,
        pub true_case: Box<HIRExpression>,
        pub false_case: Option<Box<HIRExpression>>,
    }

    impl Eq for HIRBranch {}
    impl Hash for HIRBranch {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }
    impl StructuralEq for HIRBranch {
        fn eq_ignore_id(&self, other: &Self) -> bool {
            let mut is_eq = true;
            is_eq &= (self.condition).eq_ignore_id(&other.condition);
            is_eq &= (self.true_case).eq_ignore_id(&other.true_case);
            is_eq &= (self.false_case).eq_ignore_id(&other.false_case);
            is_eq
        }
    }

    impl HIRBranch {
        pub fn if_block(condition: HIRExpression, then: HIRExpression) -> Self {
            HIRBranch {
                id: NodeId::sentinel(),
                condition: Box::new(condition),
                true_case: Box::new(then),
                false_case: None,
            }
        }
        pub fn if_else_block(
            condition: HIRExpression,
            then: HIRExpression,
            otherwise: HIRExpression,
        ) -> Self {
            HIRBranch {
                id: NodeId::sentinel(),
                condition: Box::new(condition),
                true_case: Box::new(then),
                false_case: Some(Box::new(otherwise)),
            }
        }
        pub fn wrap_in_expression(self) -> HIRExpression {
            HIRExpression {
                id: NodeId::sentinel(),
                kind: HIRExpressionKind::IfThenElse(self),
            }
        }
        pub fn wrap_in_statement(self) -> HIRStatement {
            HIRStatement {
                id: NodeId::sentinel(),
                kind: HIRStatementKind::IfThenElse(self),
            }
        }
    }
    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct HIRPatternMatchArm {
        pub id: NodeId,
        pat: HIRAssignmentPattern,
        value: Box<HIRExpression>,
    }
    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct HIRConditionMatchArm {
        pub id: NodeId,
        case: Box<HIRExpression>,
        value: Box<HIRExpression>,
    }
    /// the left hand side of an assignment
    ///
    /// The simplest is a basic identifier
    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub enum HIRAssignmentPattern {
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
        Tuple(Vec<HIRAssignmentPattern>),
    }
    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub enum HIRMatchPattern {
        /// the pattern `a => ...`
        Identifier(String),
        /// the pattern `(<pat>, <pat>, ...) => ...`
        Tuple(Vec<HIRAssignmentPattern>),

        UnionVariant(String, String, Box<HIRAssignmentPattern>),
    }

    impl std::fmt::Display for HIRAssignmentPattern {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let s = match self {
                HIRAssignmentPattern::Identifier(s) => s.clone(),
                HIRAssignmentPattern::Tuple(tups) => {
                    let s: Vec<String> = tups.iter().map(|pat| pat.to_string()).collect();
                    format!("({})", s.join(", "))
                }
            };
            write!(f, "{}", s)
        }
    }

    /// The Zea named Struct type / product type
    #[derive(Debug, Clone)]
    pub struct HIRStructDataTypeDefinition {
        pub id: NodeId,
        pub name: String,
        pub members: Vec<HIRTypedIdentifier>,
        pub reorder_fields: Option<bool>,
        pub alignment: Option<usize>,
    }
    impl HIRStructDataTypeDefinition {
        pub(crate) fn should_reorder_fields(&self) -> bool {
            self.reorder_fields.is_none_or(|b| b)
        }
    }

    pub struct HIRTaggedUnionDataTypeDefinition {
        pub name: String,
        pub members: Vec<HIRTaggedUnionVariant>,
    }

    pub enum HIRTaggedUnionVariant {
        TagVariant(String),
        DataVariant(HIRTypedIdentifier),
    }

    /// The Type that is bundled with a:
    /// - function parameter
    /// - identifier in declaration(-assignments)
    #[derive(PartialEq, Eq, Clone, Hash)]
    pub enum HIRTypeSpecifier {
        /// An aggregate DataType
        NonScalar(String),
        /// the type that a statement returns: similar to `void` or `()`
        Unit,
        /// boolean type
        Bool,
        /// Integer type with width and sign
        Integer { width: usize, signed: bool },
        /// Floating point type with width
        Float { width: usize },

        /// a pointer to a memory location containing something of the inner type
        Pointer(Box<HIRTypeSpecifier>),
        /// a pointer+length bundle of items of the inner type
        ArrayOf(Box<HIRTypeSpecifier>),
        // /// `&[<type>]`
        // Slice(Box<Type>),
        // /// `?<type>`
        // Option(Box<Type>),
        /// The diverging type, i.e. the type that `exit()` and `panic!()` return
        Never,
    }

    impl Debug for HIRTypeSpecifier {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let str = match self {
                HIRTypeSpecifier::NonScalar(typ) => typ,
                HIRTypeSpecifier::Float { width } => &format!("f{width}"),
                HIRTypeSpecifier::Integer { width, signed } => {
                    &format!("{}{width}", if *signed { 'i' } else { 'u' })
                }
                HIRTypeSpecifier::Bool => "Bool",
                HIRTypeSpecifier::ArrayOf(arr) => &format!("[{arr:?}]"),
                // Type::Option(opt) => &format!("?{opt:?}"),
                HIRTypeSpecifier::Pointer(ptr) => &format!("&{ptr:?}"),
                // Type::Slice(slice) => &format!("&[{slice:?}]"),
                HIRTypeSpecifier::Unit => "()",
                HIRTypeSpecifier::Never => "!",
            };

            write!(f, "{}", str)
        }
    }

    #[allow(non_snake_case)]
    impl HIRTypeSpecifier {
        pub const fn t_U8() -> HIRTypeSpecifier {
            Self::Integer {
                width: 8,
                signed: false,
            }
        }
        pub const fn t_U16() -> HIRTypeSpecifier {
            Self::Integer {
                width: 16,
                signed: false,
            }
        }
        pub const fn t_U32() -> HIRTypeSpecifier {
            Self::Integer {
                width: 32,
                signed: false,
            }
        }
        pub const fn t_U64() -> HIRTypeSpecifier {
            Self::Integer {
                width: 64,
                signed: false,
            }
        }
        pub const fn t_I8() -> HIRTypeSpecifier {
            Self::Integer {
                width: 8,
                signed: true,
            }
        }
        pub const fn t_I16() -> HIRTypeSpecifier {
            Self::Integer {
                width: 16,
                signed: true,
            }
        }
        pub const fn t_I32() -> HIRTypeSpecifier {
            Self::Integer {
                width: 32,
                signed: true,
            }
        }
        pub const fn t_I64() -> HIRTypeSpecifier {
            Self::Integer {
                width: 64,
                signed: true,
            }
        }

        pub const fn t_F32() -> HIRTypeSpecifier {
            Self::Float { width: 32 }
        }
        pub const fn t_F64() -> HIRTypeSpecifier {
            Self::Float { width: 64 }
        }

        pub const fn t_Bool() -> Self {
            Self::Bool
        }
        pub const fn t_Unit() -> Self {
            HIRTypeSpecifier::Unit
        }

        pub const fn t_Never() -> Self {
            HIRTypeSpecifier::Never
        }
    }
    #[derive(Debug, Eq, PartialEq, Hash, Clone, ASTStructuralEq)]
    pub struct HIRTypedIdentifier {
        pub name: String,
        pub typ: HIRTypeSpecifier,
    }

    impl HIRTypedIdentifier {
        pub fn new(typ: HIRTypeSpecifier, name: impl Into<String>) -> Self {
            Self {
                typ,
                name: name.into(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantToStr)]
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

#[derive(Clone, Debug)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}
#[cfg(test)]
pub(crate) mod test_ast_macros {
    macro_rules! label_ast {
        (fresh $ast:expr) => {{
            use crate::zea::visitors::altering::{BareNodeLabeler, NodeLabeler};
            use crate::zea::Module;

            let mut node_labeler = BareNodeLabeler::new();
            let mut ast = $ast;
            ast.accept_sentinel_labeler(&mut node_labeler);
            (ast, node_labeler)
        }};
        (using $l:expr ; $ast:expr) => {{
            let mut ast = $ast;
            let mut l = $l;
            ast.accept_sentinel_labeler(&mut l);
            (ast, l)
        }};
    }
    pub(crate) use label_ast;
    macro_rules! block {
        {} => {
           {
               BlockExpression {
                    id: NodeId::sentinel(),
                    statements: vec![],
                   last: Expression::unit()
               }
           }
        };
        {$($e:expr);* ; $last:expr} => {
           {
               let last: Expression = $last;
               BlockExpression {
                    id: NodeId::sentinel(),
                    statements: vec![$($e),*]
                    last
               }
           }
        };
        {exp $($e:expr);+ $(;)?} => {
            {
                let be = crate::zea::BlockExpander::new();
                let mut b: Statement = stmt!(block
                StatementBlock {
                    id: NodeId::sentinel(),
                    statements: vec![$($e),+]
               };);
                b.accept_block_expander(&mut be);
                let StatementKind::ExpandedBlock(eb) = b.kind else {panic!("expected expanded block")};
                eb
            }
        }
    }

    pub(crate) use block;

    macro_rules! stmt {
        (ret $e:expr) => {
            {use crate::zea::{Statement,StatementKind, NodeId};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Return($e)
            }
        }};
        (block $e:expr) => {
            {
                use crate::zea::{Statement, StatementKind};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Block($e)
            }
        }};
        (tail $e:expr) => {
            {use crate::zea::{Statement,StatementKind, NodeId};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::BlockTail($e)
            }
        }};
        (call $name:ident ($($e:expr),*)) => {
            {
                use crate::zea::{Statement, StatementKind, FunctionCall}
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::FunctionCall(FunctionCall {
                    id: NodeId::sentinel(),
                    subject: $name,
                    args: vec![$($e),*]
                })
            }
        }};

        (init $p:expr ;= $val:expr) => {
            {
                use crate::zea::{AssignmentPattern,InitializationBlock,Statement,StatementKind};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Initialization(InitializationBlock {
                    id: NodeId::sentinel(),
                    kind: InitializationKind::Packed(
                        PackedInitialization {
                    assignee: $p,
                    typ: None,
                    value: $val,
                        }
                    )
                })
            }
        }};
    }
    pub(crate) use stmt;
    macro_rules! init {
        ($p:expr ;= $val:expr) => {{
            use crate::zea::InitializationBlock;
            InitializationBlock {
                id: NodeId::sentinel(),
                kind: InitializationKind::Packed(PackedInitialization {
                    assignee: $p,
                    typ: None,
                    value: $val,
                }),
            }
        }};
    }
    pub(crate) use init;

    // generated by claude code, prompt:
    // "can you make the pat macro a tt muncher
    // that accepts things like (a,(b,c)) and converts it to a nested assignment pattern"
    // + pat macro
    macro_rules! pat {
    // Single identifier — base case
    ($i:ident) => {
        AssignmentPattern::Identifier(String::from(stringify!($i)))
    };
    // Outer tuple — kick off the muncher with an empty accumulator
    (($($t:tt)*)) => {
        pat!(@munch [] $($t)*)
    };
    // Muncher: accumulator is complete, nothing left to consume
    (@munch [$($acc:expr),*]) => {
        AssignmentPattern::Tuple(vec![$($acc),*])
    };
    // Muncher: next item is a nested tuple, more items follow
    (@munch [$($acc:expr),*] ($($inner:tt)*), $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))] $($rest)*)
    };
    // Muncher: next item is a nested tuple, nothing follows
    (@munch [$($acc:expr),*] ($($inner:tt)*)) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))])
    };
    // Muncher: next item is an identifier, more items follow
    (@munch [$($acc:expr),*] $i:ident, $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!($i)] $($rest)*)
    };
    // Muncher: next item is an identifier, nothing follows
    (@munch [$($acc:expr),*] $i:ident) => {
        pat!(@munch [$($acc,)* pat!($i)])
    };

}
    pub(crate) use pat;
    macro_rules! expr {
        (ident $($l:tt)+) => {{
            use crate::zea::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::UnScopedIdent(String::from(stringify!($($l)+))),
            }
        }};
        (litint $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::IntegerLiteral($l),
            }
        }};
        (litfloat $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::FloatLiteral($l),
            }
        }};
        (litbool $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::BoolLiteral($l),
            }
        }};
        (litstr $l:literal) => {
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::StringLiteral(stringify!($l)),
            }
        };
        (unit) => {{
            use crate::zea::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::Unit,
            }
        }};
        (block $block:expr) => {{
            use crate::zea::{Expression,ExpressionKind,StatementBlock};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::Block(Box::new($block))
            }
            }
        }
    }
    pub(crate) use expr;

    macro_rules! zea_module {
        (imports {$($imp:ident),* $(,)?}
         exports {$($exp:ident),* $(,)?}
         globs   {$($glob:expr);* $(;)?}
         funcs   {$($func:expr);* $(;)?}
         structs {$($struct_def:expr);* $(;)?}
        ) => {
            {
                use crate::zea::{Module, NodeId};
                Module {
                    id: NodeId::sentinel(),
                    imports: vec![$(String::from(stringify!($imp))),*],
                    exports: vec![$(String::from(stringify!($exp))),*],
                    global_vars: vec![$($glob),*],
                    functions: vec![$($func),*],
                    struct_definitions: vec![$($struct_def),*]
                }
            }
        };
        (imports {$($imp:ident),* $(,)?}
         globs   {$($glob:expr);* $(;)?}
         funcs   {$($func:expr);* $(;)?}
        ) => {
            {
                use crate::zea::{Module, NodeId};
                Module {
                    id: NodeId::sentinel(),
                    imports: vec![$(String::from(stringify!($imp))),*],
                    exports: vec![],
                    global_vars: vec![$($glob),*],
                    functions: vec![$($func),*],
                    struct_definitions: vec![]
                }
            }
        };
    }
    pub(crate) use zea_module;
    macro_rules! func {
        {$name:ident ( $($arg:ident: $typ:expr),* ) -> $ret:expr; $body:expr } => {
            {
                use crate::zea::{Function, TypedIdentifier};
                let args = vec![$(
                TypedIdentifier(String::from(stringify!($arg)), $typ)
                ),*];
                Function {
                    id: NodeId::sentinel(),
                    name: String::from(stringify!($name)),
                    params:args,
                    returns: $ret,
                    body: $body,
                }
            }
        };
    }
    pub(crate) use func;
    macro_rules! ztyp {
        (U8) => {
            {
            use crate::zea::hir_nodes::HIRTypeSpecifier;
            HIRTypeSpecifier::t_U8()
        }
    };
        (I8) => {{
            use crate::zea::hir_nodes::HIRTypeSpecifier;
            HIRTypeSpecifier::t_I8()
        }
    };
        ($t:ident) => {
            {
            use crate::zea::hir_nodes::HIRTypeSpecifier;
                HIRTypeSpecifier::NonScalar(String::from(stringify!($t)))
            }
        };
        (*$($t:tt)+) => {
            {
            use crate::zea::hir_nodes::HIRTypeSpecifier;
                HIRTypeSpecifier::Pointer(Box::new(ztyp!($($t)+)))
            }
        };
        ([ ]$($t:tt)+) => {
            {
            use crate::zea::hir_nodes::HIRTypeSpecifier;
                HIRTypeSpecifier::ArrayOf(Box::new(ztyp!($($t)+)))
            }
        };
    }
    pub(crate) use ztyp;
}

pub use crate::zea::visitors::altering::{BareNodeLabeler, NodeLabeler};
pub use crate::zea::visitors::annotating::HIRScopedIdentifier;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use zea_internal_macros::{ASTStructuralEq, VariantToStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct NodeId(usize);

impl NodeId {
    pub const fn sentinel() -> Self {
        Self(0)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }
}
impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

struct ZeaNodeQuery {
    id: NodeId,
}
impl ZeaNodeQuery {
    pub fn query_hir_node_with_id(id: NodeId, module: &HIRModule) -> Option<HIRASTNode> {
        let mut s = Self { id };
        match s.visit_module(module) {
            Ok(Some(n)) => Some(n),
            _ => None,
        }
    }
}

fn float_total_cmp(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else {
        a == b
    }
}
