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
use crate::traits::StructuralEq;
use crate::zea::ipr::IPRASTNode;
use crate::zea::ipr::IPRModule;
use crate::zea::visitors::IPRVisitor;
use arbitrary::{Arbitrary, Unstructured};

mod typecheck;
pub use typecheck::typecheck_module;

mod impls;
pub mod visitors;

pub mod thr;
pub mod ipr {
    use arbitrary::{Arbitrary, Unstructured};
    use std::{
        char,
        fmt::{Debug, Formatter},
        hash::{Hash, Hasher},
    };

    use zea_internal_macros::{ASTStructuralEq, VariantToStr};

    use crate::{
        traits::StructuralEq,
        zea::{
            BareNodeLabeler, BinOp, IPRScopedIdentifier, NodeLabeler, UnOp, ZeaNodeQuery,
            visitors::{
                IPRTransfomer, IPRVisitor,
                altering::{AssignmentSimplifier, IdentifierScoper, InsertImplicitMainReturn},
            },
        },
    };

    use super::NodeId;

    #[derive(Debug, Arbitrary)]
    pub enum IPRASTNode {
        Module(IPRModule),
        Function(IPRFunction),
        Block(IPRBlockExpression),
        Branch(IPRBranch),
        Expression(IPRExpression),
        Statement(IPRStatement),
        Call(IPRFunctionCall),
        FuncParam(IPRFuncParam),
        Init(IPRSimpleInitialization),
    }

    impl IPRASTNode {
        fn id(&self) -> NodeId {
            match self {
                IPRASTNode::Module(m) => m.id,
                IPRASTNode::Function(f) => f.id,
                IPRASTNode::Block(b) => b.id,
                IPRASTNode::Branch(b) => b.id,
                IPRASTNode::Expression(e) => e.id,
                IPRASTNode::Statement(s) => s.id,
                IPRASTNode::Call(c) => c.id,
                IPRASTNode::FuncParam(p) => p.id,
                IPRASTNode::Init(i) => i.id,
            }
        }
    }
    impl From<IPRSimpleInitialization> for IPRASTNode {
        fn from(v: IPRSimpleInitialization) -> Self {
            Self::Init(v)
        }
    }

    impl From<IPRFuncParam> for IPRASTNode {
        fn from(v: IPRFuncParam) -> Self {
            Self::FuncParam(v)
        }
    }

    impl From<IPRFunctionCall> for IPRASTNode {
        fn from(v: IPRFunctionCall) -> Self {
            Self::Call(v)
        }
    }

    impl From<IPRStatement> for IPRASTNode {
        fn from(v: IPRStatement) -> Self {
            Self::Statement(v)
        }
    }

    impl From<IPRExpression> for IPRASTNode {
        fn from(v: IPRExpression) -> Self {
            Self::Expression(v)
        }
    }

    impl From<IPRBranch> for IPRASTNode {
        fn from(v: IPRBranch) -> Self {
            Self::Branch(v)
        }
    }

    impl From<IPRBlockExpression> for IPRASTNode {
        fn from(v: IPRBlockExpression) -> Self {
            Self::Block(v)
        }
    }

    impl From<IPRFunction> for IPRASTNode {
        fn from(v: IPRFunction) -> Self {
            Self::Function(v)
        }
    }

    impl From<IPRModule> for IPRASTNode {
        fn from(v: IPRModule) -> Self {
            Self::Module(v)
        }
    }

    impl IPRASTNode {
        pub fn as_module(&self) -> Option<&IPRModule> {
            if let Self::Module(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_function(&self) -> Option<&IPRFunction> {
            if let Self::Function(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_block(&self) -> Option<&IPRBlockExpression> {
            if let Self::Block(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_branch(&self) -> Option<&IPRBranch> {
            if let Self::Branch(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_expression(&self) -> Option<&IPRExpression> {
            if let Self::Expression(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_statement(&self) -> Option<&IPRStatement> {
            if let Self::Statement(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_call(&self) -> Option<&IPRFunctionCall> {
            if let Self::Call(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }

    #[derive(Default, Debug, Clone, Arbitrary)]
    pub struct IPRModule {
        pub id: NodeId,
        pub imports: Vec<String>,
        pub exports: Vec<String>,
        pub global_vars: Vec<IPRInitializationBlock>,
        pub functions: Vec<IPRFunction>,
        pub struct_definitions: Vec<IPRStructDataTypeDefinition>,
        pub name: String,
    }
    impl IPRModule {
        pub(crate) fn get_main(&mut self) -> Option<&mut IPRFunction> {
            self.functions.iter_mut().find(|f| f.name == "main")
        }

        pub fn label_nodes(&mut self) -> BareNodeLabeler {
            let mut l = BareNodeLabeler::new();
            l.visit_module(self).unwrap();
            l
        }

        pub fn insert_implicit_main_return(
            &mut self,
            labeler: impl NodeLabeler,
        ) -> impl NodeLabeler {
            let mut transformer = InsertImplicitMainReturn::labeler_from(labeler);
            transformer.visit_module(self).unwrap();
            transformer
        }

        pub fn visit_self_with<V: IPRVisitor + NodeLabeler>(
            &mut self,
            labeler: impl NodeLabeler,
        ) -> Result<V, V::VisitorError> {
            let mut v: V = labeler.labeler_into();
            v.visit_module(self)?;
            Ok(v)
        }

        pub fn transform_self_with<V: IPRTransfomer + NodeLabeler>(
            &mut self,
            labeler: impl NodeLabeler,
        ) -> Result<V, V::TransformerError> {
            let mut v: V = labeler.labeler_into();
            v.visit_module(self).map(|_| v)
        }

        pub fn simplify_assignments_after(
            &mut self,
            labeler: impl NodeLabeler,
        ) -> AssignmentSimplifier {
            self.transform_self_with(labeler).unwrap()
        }
        pub fn scope_idents(mut self) -> (Self, IdentifierScoper) {
            let mut scoper = IdentifierScoper::new(&self);
            match scoper.visit_module(&mut self) {
                Ok(_) => (self, scoper),
                Err(e) => panic!("{e:?}"),
            }
        }
    }
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRFuncParam {
        pub id: NodeId,
        pub typ: IPRTypeSpecifier,
        pub name: String,
    }

    /// A top-level function definition
    ///
    /// Function may be defined only once within a module.
    /// Functions may be imported as many times as needed.
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRFunction {
        pub id: NodeId,
        pub name: String,
        pub params: Vec<IPRFuncParam>,
        pub returns: IPRTypeSpecifier,
        pub body: IPRBlockExpression,
    }

    #[derive(Debug, Clone)]
    pub struct HoistedFunctionSignature {
        pub id: NodeId,
        pub name: String,
        pub args: Vec<IPRFuncParam>,
        pub returns: IPRTypeSpecifier,
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRStatement {
        pub id: NodeId,
        pub kind: IPRStatementKind,
    }

    #[derive(Debug, Clone, VariantToStr, Arbitrary)]
    pub enum IPRStatementKind {
        // initial pass
        /// Variable initialization
        Initialization(IPRInitializationBlock),
        /// Variable Reassignment
        Reassignment(IPRReassignment),
        FunctionCall(IPRFunctionCall),
        /// Control-flow return
        Return(IPRExpression),

        // CondMatch(Box<ConditionMatch>),

        // after expansion
        Block(IPRBlockExpression),
        IfThenElse(IPRBranch),
    }

    /// A packed or unpacked initialisation
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRInitializationBlock {
        pub id: NodeId,
        pub kind: IPRInitializationKind,
    }
    impl StructuralEq for IPRInitializationBlock {
        fn eq_ignore_id(&self, other: &Self) -> bool {
            let mut is_eq = true;
            is_eq &= (self.kind).eq_ignore_id(&other.kind);
            is_eq
        }
    }

    impl IPRInitializationBlock {
        pub fn packed(
            typ: Option<IPRTypeSpecifier>,
            assignee: IPRAssignmentPattern,
            value: IPRExpression,
        ) -> Self {
            Self {
                id: NodeId::sentinel(),
                kind: IPRInitializationKind::Packed(IPRPackedInitialization {
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
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRPackedInitialization {
        pub typ: Option<IPRTypeSpecifier>,
        pub assignee: IPRAssignmentPattern,
        pub value: IPRExpression,
    }

    impl IPRPackedInitialization {
        pub fn untyped(assignee: IPRAssignmentPattern, value: IPRExpression) -> Self {
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
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRSimpleInitialization {
        pub id: NodeId,
        pub typ: Option<IPRTypeSpecifier>,
        pub assignee: String,
        pub value: IPRExpression,
    }

    impl IPRSimpleInitialization {
        pub fn untyped(assignee: &str, value: IPRExpression) -> Self {
            Self {
                id: NodeId::sentinel(),
                assignee: assignee.to_string(),
                value,
                typ: None,
            }
        }

        pub fn with_id(mut self, id: NodeId) -> IPRSimpleInitialization {
            self.id = id;
            self
        }
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRPartiallyUnpackedInitialization {
        pub temporary: IPRSimpleInitialization,
        pub unpacked_assignments: Vec<IPRInitializationBlock>,
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub enum IPRInitializationKind {
        Packed(IPRPackedInitialization),
        Unpacked(Vec<IPRSimpleInitialization>),
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRReassignment {
        pub id: NodeId,
        pub assignee: String,
        pub value: IPRExpression,
    }

    impl IPRReassignment {
        pub fn wrap_in_statement(self) -> IPRStatement {
            IPRStatement {
                id: NodeId::sentinel(),
                kind: IPRStatementKind::Reassignment(self),
            }
        }
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRFunctionCall {
        pub id: NodeId,
        pub subject: Box<IPRExpression>,
        pub args: Vec<IPRExpression>,
    }

    impl IPRFunctionCall {
        pub fn wrap_in_statement(self) -> IPRStatement {
            IPRStatement {
                id: NodeId::sentinel(),
                kind: IPRStatementKind::FunctionCall(self),
            }
        }

        pub fn wrap_in_expression(self) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::FunctionCall(self),
            }
        }
    }

    #[derive(Clone, Debug, Arbitrary)]
    pub struct IPRBlockExpression {
        /// The label that the block expression has its value assigned to
        /// i.e. `__block0`, `__block1` etc.
        /// This label must be unique to the scope of the function in which it exists
        pub id: NodeId,
        pub statements: Vec<IPRStatement>,
        pub tail: IPRExpression,
    }

    impl IPRBlockExpression {
        pub fn wrap_in_statement(self) -> IPRStatement {
            IPRStatement {
                id: NodeId::sentinel(),
                kind: IPRStatementKind::Block(self),
            }
        }
        pub fn wrap_in_expression(self) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::Block(Box::new(self)),
            }
        }
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRExpression {
        pub id: NodeId,
        pub kind: IPRExpressionKind,
    }

    impl IPRExpression {
        pub fn with_id(mut self, id: NodeId) -> Self {
            self.id = id;
            self
        }
    }

    impl IPRExpression {
        pub fn tuple_member_access(e: IPRExpression, field: usize) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::MemberAccess(Box::new(e), format!("_{field}")),
            }
        }

        pub fn ident(ident: String) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::UnScopedIdent(ident),
            }
        }
        pub fn scoped_local(ident: String, origin: NodeId) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::ScopedIdent(IPRScopedIdentifier::local(origin, ident)),
            }
        }

        pub fn wrap_in_return_statement(self) -> IPRStatement {
            IPRStatement {
                id: NodeId::sentinel(),
                kind: IPRStatementKind::Return(self),
            }
        }

        pub fn wrap_lit_int(i: usize) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::IntegerLiteral(i),
            }
        }
        pub fn wrap_lit_float(f: f64) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::FloatLiteral(f),
            }
        }

        pub fn wrap_lit_bool(b: bool) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::BoolLiteral(b),
            }
        }

        pub fn wrap_ident(ident: String) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::UnScopedIdent(ident),
            }
        }
    }

    #[derive(Debug, Clone, VariantToStr, Arbitrary)]
    pub enum IPRExpressionKind {
        // initial pass
        Unit,
        IntegerLiteral(usize),
        BoolLiteral(bool),
        FloatLiteral(f64),
        StringLiteral(String),
        UnScopedIdent(String),
        ScopedIdent(IPRScopedIdentifier),
        FunctionCall(IPRFunctionCall),
        BinOpExpr(BinOp, Box<IPRExpression>, Box<IPRExpression>),
        UnOpExpr(UnOp, Box<IPRExpression>),
        MemberAccess(Box<IPRExpression>, String),
        IfThenElse(IPRBranch),

        // PatternMatch(PatternMatch),
        // ConditionMatch(ConditionMatch),
        // IfThenElse(IfThenElse),

        // after expansion
        Block(Box<IPRBlockExpression>),
    }

    impl IPRExpression {
        pub const fn unit() -> Self {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::Unit,
            }
        }

        pub fn binop(op: BinOp, l: IPRExpression, r: IPRExpression) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::BinOpExpr(op, Box::new(l), Box::new(r)),
            }
        }
        pub fn unop(op: UnOp, e: IPRExpression) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::UnOpExpr(op, Box::new(e)),
            }
        }

        pub fn member_access(data: IPRExpression, member: String) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::MemberAccess(Box::new(data), member),
            }
        }
    }

    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct ConditionMatch {
        pub id: NodeId,
        conditions: Vec<IPRConditionMatchArm>,
    }

    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct PatternMatch {
        pub id: NodeId,
        patterns: Vec<IPRPatternMatchArm>,
        subject: Box<IPRExpression>,
    }

    #[derive(Clone, Debug, Arbitrary)]
    pub struct IPRBranch {
        pub id: NodeId,
        pub condition: Box<IPRExpression>,
        pub true_case: Box<IPRExpression>,
        pub false_case: Option<Box<IPRExpression>>,
    }

    impl Eq for IPRBranch {}
    impl Hash for IPRBranch {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }
    impl StructuralEq for IPRBranch {
        fn eq_ignore_id(&self, other: &Self) -> bool {
            let mut is_eq = true;
            is_eq &= (self.condition).eq_ignore_id(&other.condition);
            is_eq &= (self.true_case).eq_ignore_id(&other.true_case);
            is_eq &= (self.false_case).eq_ignore_id(&other.false_case);
            is_eq
        }
    }

    impl IPRBranch {
        pub fn if_block(condition: IPRExpression, then: IPRExpression) -> Self {
            IPRBranch {
                id: NodeId::sentinel(),
                condition: Box::new(condition),
                true_case: Box::new(then),
                false_case: None,
            }
        }
        pub fn if_else_block(
            condition: IPRExpression,
            then: IPRExpression,
            otherwise: IPRExpression,
        ) -> Self {
            IPRBranch {
                id: NodeId::sentinel(),
                condition: Box::new(condition),
                true_case: Box::new(then),
                false_case: Some(Box::new(otherwise)),
            }
        }
        pub fn wrap_in_expression(self) -> IPRExpression {
            IPRExpression {
                id: NodeId::sentinel(),
                kind: IPRExpressionKind::IfThenElse(self),
            }
        }
        pub fn wrap_in_statement(self) -> IPRStatement {
            IPRStatement {
                id: NodeId::sentinel(),
                kind: IPRStatementKind::IfThenElse(self),
            }
        }
    }
    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct IPRPatternMatchArm {
        pub id: NodeId,
        pat: IPRAssignmentPattern,
        value: Box<IPRExpression>,
    }
    #[derive(Clone, Debug, ASTStructuralEq)]
    pub struct IPRConditionMatchArm {
        pub id: NodeId,
        case: Box<IPRExpression>,
        value: Box<IPRExpression>,
    }
    /// the left hand side of an assignment
    ///
    /// The simplest is a basic identifier
    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub enum IPRAssignmentPattern {
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
        Tuple(Vec<IPRAssignmentPattern>),
    }
    fn arbitrary_ascii_printable_string<'a>(u: &mut Unstructured<'a>) -> arbitrary::Result<String> {
        let count = u.arbitrary_len::<char>()?.max(1);
        let mut buffer = String::with_capacity(count);
        for _ in 0..count {
            let s = u.int_in_range('!' as u32..='~' as u32)?;
            let s: char = char::from_u32(s).unwrap();
            buffer.push(s);
        }
        Ok(buffer)
    }
    impl<'a> Arbitrary<'a> for IPRAssignmentPattern {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let choice = u.choose_index(2)?;
            if choice == 0 {
                let s = arbitrary_ascii_printable_string(u)?;
                Ok(IPRAssignmentPattern::Identifier(s))
            } else {
                let item_count = u.arbitrary_len::<IPRAssignmentPattern>()?.max(2);
                let mut v = Vec::with_capacity(item_count);
                for _ in 0..item_count {
                    v.push(IPRAssignmentPattern::arbitrary(u)?);
                }
                Ok(IPRAssignmentPattern::Tuple(v))
            }
        }
        fn try_size_hint(
            depth: usize,
        ) -> arbitrary::Result<(usize, Option<usize>), arbitrary::MaxRecursionReached> {
            String::try_size_hint(depth)
        }
    }

    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub enum IPRMatchPattern {
        /// the pattern `a => ...`
        Identifier(String),
        /// the pattern `(<pat>, <pat>, ...) => ...`
        Tuple(Vec<IPRAssignmentPattern>),

        UnionVariant(String, String, Box<IPRAssignmentPattern>),
    }

    impl std::fmt::Display for IPRAssignmentPattern {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let s = match self {
                IPRAssignmentPattern::Identifier(s) => s.clone(),
                IPRAssignmentPattern::Tuple(tups) => {
                    let s: Vec<String> = tups.iter().map(|pat| pat.to_string()).collect();
                    format!("({})", s.join(", "))
                }
            };
            write!(f, "{}", s)
        }
    }

    /// The Zea named Struct type / product type
    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRStructDataTypeDefinition {
        pub id: NodeId,
        pub name: String,
        pub members: Vec<IPRTypedIdentifier>,
        pub reorder_fields: Option<bool>,
        pub alignment: Option<usize>,
    }
    impl IPRStructDataTypeDefinition {
        pub(crate) fn should_reorder_fields(&self) -> bool {
            self.reorder_fields.is_none_or(|b| b)
        }
    }

    pub struct IPRTaggedUnionDataTypeDefinition {
        pub name: String,
        pub members: Vec<IPRTaggedUnionVariant>,
    }

    pub enum IPRTaggedUnionVariant {
        TagVariant(String),
        DataVariant(IPRTypedIdentifier),
    }

    /// The Type that is bundled with a:
    /// - function parameter
    /// - identifier in declaration(-assignments)
    #[derive(PartialEq, Eq, Clone, Hash, Arbitrary)]
    pub enum IPRTypeSpecifier {
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
        Pointer(Box<IPRTypeSpecifier>),
        /// a pointer+length bundle of items of the inner type
        ArrayOf(Box<IPRTypeSpecifier>),
        // /// `&[<type>]`
        // Slice(Box<Type>),
        // /// `?<type>`
        // Option(Box<Type>),
        /// The diverging type, i.e. the type that `exit()` and `panic!()` return
        Never,
    }

    impl Debug for IPRTypeSpecifier {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let str = match self {
                IPRTypeSpecifier::NonScalar(typ) => typ,
                IPRTypeSpecifier::Float { width } => &format!("f{width}"),
                IPRTypeSpecifier::Integer { width, signed } => {
                    &format!("{}{width}", if *signed { 'i' } else { 'u' })
                }
                IPRTypeSpecifier::Bool => "Bool",
                IPRTypeSpecifier::ArrayOf(arr) => &format!("[{arr:?}]"),
                // Type::Option(opt) => &format!("?{opt:?}"),
                IPRTypeSpecifier::Pointer(ptr) => &format!("&{ptr:?}"),
                // Type::Slice(slice) => &format!("&[{slice:?}]"),
                IPRTypeSpecifier::Unit => "()",
                IPRTypeSpecifier::Never => "!",
            };

            write!(f, "{}", str)
        }
    }

    #[allow(non_snake_case)]
    impl IPRTypeSpecifier {
        pub const fn t_U8() -> IPRTypeSpecifier {
            Self::Integer {
                width: 8,
                signed: false,
            }
        }
        pub const fn t_U16() -> IPRTypeSpecifier {
            Self::Integer {
                width: 16,
                signed: false,
            }
        }
        pub const fn t_U32() -> IPRTypeSpecifier {
            Self::Integer {
                width: 32,
                signed: false,
            }
        }
        pub const fn t_U64() -> IPRTypeSpecifier {
            Self::Integer {
                width: 64,
                signed: false,
            }
        }
        pub const fn t_I8() -> IPRTypeSpecifier {
            Self::Integer {
                width: 8,
                signed: true,
            }
        }
        pub const fn t_I16() -> IPRTypeSpecifier {
            Self::Integer {
                width: 16,
                signed: true,
            }
        }
        pub const fn t_I32() -> IPRTypeSpecifier {
            Self::Integer {
                width: 32,
                signed: true,
            }
        }
        pub const fn t_I64() -> IPRTypeSpecifier {
            Self::Integer {
                width: 64,
                signed: true,
            }
        }

        pub const fn t_F32() -> IPRTypeSpecifier {
            Self::Float { width: 32 }
        }
        pub const fn t_F64() -> IPRTypeSpecifier {
            Self::Float { width: 64 }
        }

        pub const fn t_Bool() -> Self {
            Self::Bool
        }
        pub const fn t_Unit() -> Self {
            IPRTypeSpecifier::Unit
        }

        pub const fn t_Never() -> Self {
            IPRTypeSpecifier::Never
        }
        pub fn inner_from_str(s: &str) -> Self {
            match s {
                "U8" => Self::t_U8(),
                "I8" => Self::t_I8(),
                "U16" => Self::t_U16(),
                "I16" => Self::t_I16(),
                "U32" => Self::t_U32(),
                "I32" => Self::t_I32(),
                "U64" => Self::t_U64(),
                "I64" => Self::t_I64(),
                "F32" => Self::t_F32(),
                "F64" => Self::t_F64(),
                "()" => Self::t_Unit(),
                "!" => Self::t_Never(),
                "Bool" => Self::t_Bool(),
                s => Self::NonScalar(s.to_string()),
            }
        }
    }
    #[derive(Debug, Eq, PartialEq, Hash, Clone, ASTStructuralEq, Arbitrary)]
    pub struct IPRTypedIdentifier {
        pub name: String,
        pub typ: IPRTypeSpecifier,
    }

    impl IPRTypedIdentifier {
        pub fn new(typ: IPRTypeSpecifier, name: impl Into<String>) -> Self {
            Self {
                typ,
                name: name.into(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantToStr, Arbitrary)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantToStr, Arbitrary)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}
#[cfg(test)]
pub(crate) mod test_ast_macros {
    macro_rules! label_ast {
        (fresh $ast:expr) => {{
            use crate::zea::Module;
            use crate::zea::visitors::altering::{BareNodeLabeler, NodeLabeler};

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
            use crate::zea::immediate_parsed_representation::IPRTypeSpecifier;
            IPRTypeSpecifier::t_U8()
        }
    };
        (I8) => {{
            use crate::zea::immediate_parsed_representation::IPRTypeSpecifier;
            IPRTypeSpecifier::t_I8()
        }
    };
        ($t:ident) => {
            {
            use crate::zea::immediate_parsed_representation::IPRTypeSpecifier;
                IPRTypeSpecifier::NonScalar(String::from(stringify!($t)))
            }
        };
        (*$($t:tt)+) => {
            {
            use crate::zea::immediate_parsed_representation::IPRTypeSpecifier;
                IPRTypeSpecifier::Pointer(Box::new(ztyp!($($t)+)))
            }
        };
        ([ ]$($t:tt)+) => {
            {
            use crate::zea::immediate_parsed_representation::IPRTypeSpecifier;
                IPRTypeSpecifier::ArrayOf(Box::new(ztyp!($($t)+)))
            }
        };
    }
    pub(crate) use ztyp;
}

pub use crate::zea::visitors::altering::{BareNodeLabeler, NodeLabeler};
pub use crate::zea::visitors::annotating::IPRScopedIdentifier;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use zea_internal_macros::{ASTStructuralEq, VariantToStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct NodeId(usize);
impl<'a> Arbitrary<'a> for NodeId {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let _ = u;
        Ok(NodeId::sentinel())
    }
}

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

/// This visitor provides a way to query a node by its id, the returned node is of type [`IPRASTNode`],
/// which provides methods to destructure it into its inner node.
/// This visitor is really only useful if you know the type of the node beforehands.
pub struct ZeaNodeQuery {
    id: NodeId,
}
impl ZeaNodeQuery {
    pub fn query_ipr_node(id: NodeId, module: &IPRModule) -> Option<IPRASTNode> {
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
