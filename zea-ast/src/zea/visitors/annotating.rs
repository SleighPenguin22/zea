use crate::zea::visitors::altering::AcceptsBlockExpander;
use crate::zea::{
    AssignmentPattern, BinOp, ExpandedBlockExpr, Expression, ExpressionKind, Function,
    FunctionCall, IfThenElse, InitialisationKind, Initialization, Module, PackedInitialisation,
    PartiallyUnpackedInitialisation, Reassignment, Statement, StatementBlock, StatementKind,
    StructDataTypeDefinition, TaggedUnionDataTypeDefinition, UnOp, UnpackedInitialisation,
};
use indexmap::{IndexMap, IndexSet};
use std::collections::HashSet;

pub trait IntroducesFreshIdentifiers {
    /// Get all identifiers introduced in the current scope
    ///
    /// Note that this should return only identifiers that are newly introduced in the current scope,
    /// including shadowed ones.
    ///
    /// In general, the only node that introduces identifiers is an initialization,
    /// and a function definition.
    fn get_introduced_identifiers(&self) -> IndexSet<String>;
}
impl IntroducesFreshIdentifiers for Initialization {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        match &self.kind {
            InitialisationKind::Packed(p) => p.get_introduced_identifiers(),
            InitialisationKind::PartiallyUnpacked(p) => p.get_introduced_identifiers(),
        }
    }
}

impl IntroducesFreshIdentifiers for PackedInitialisation {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        self.assignee.get_introduced_identifiers()
    }
}
impl IntroducesFreshIdentifiers for AssignmentPattern {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        match self {
            AssignmentPattern::Identifier(s) => IndexSet::from_iter([s.clone()]),
            AssignmentPattern::Tuple(t) => t
                .iter()
                .flat_map(|assignee| assignee.get_introduced_identifiers())
                .collect(),
        }
    }
}
impl IntroducesFreshIdentifiers for UnpackedInitialisation {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        IndexSet::from_iter([self.assignee.clone()])
    }
}

impl IntroducesFreshIdentifiers for PartiallyUnpackedInitialisation {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        let mut temp_id = self.temporary.get_introduced_identifiers();
        let unpacked_ids: IndexSet<String> = self
            .unpacked_assignments
            .iter()
            .flat_map(|init| init.get_introduced_identifiers())
            .collect();
        temp_id.extend(unpacked_ids);
        temp_id
    }
}

impl IntroducesFreshIdentifiers for Statement {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        match &self.kind {
            StatementKind::Initialisation(i) => i.get_introduced_identifiers(),
            _ => IndexSet::default(),
        }
    }
}

impl IntroducesFreshIdentifiers for StatementBlock {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        self.statements
            .iter()
            .flat_map(|stmt| stmt.get_introduced_identifiers())
            .collect()
    }
}

impl IntroducesFreshIdentifiers for ExpandedBlockExpr {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        self.statements
            .iter()
            .flat_map(|stmt| stmt.get_introduced_identifiers())
            .collect()
        // the only way self.last can introduce a new ident
        // is through a block which is a deeper scope; not in this scope.
    }
}

impl IntroducesFreshIdentifiers for Function {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        IndexSet::from_iter([self.name.clone()])
    }
}
impl IntroducesFreshIdentifiers for Module {
    fn get_introduced_identifiers(&self) -> IndexSet<String> {
        self.get_globally_scoped_identifiers()
    }
}

#[derive(Default, Clone)]
/// A scope that contains both inherited and introduced identifiers
pub struct NodeScope {
    inherited: IndexSet<String>,
    introduced: IndexSet<String>,
}

impl NodeScope {
    pub fn from_inherited(inherited: IndexSet<String>) -> Self {
        Self {
            inherited,
            introduced: IndexSet::new(),
        }
    }

    pub fn inherit_from(&mut self, scope: NodeScope) {
        self.inherited.extend(scope.into_union());
    }

    pub fn from_introduced(introduced: IndexSet<String>) -> Self {
        Self {
            introduced,
            inherited: IndexSet::new(),
        }
    }
    pub fn append_introduced(&mut self, ident: &str) {
        self.introduced.insert(ident.to_owned());
    }
    pub fn extend_introduced(&mut self, idents: IndexSet<String>) {
        self.introduced.extend(idents)
    }
    pub fn append_inherited(&mut self, ident: &str) {
        self.inherited.insert(ident.to_owned());
    }
    /// extend inherited identifiers with
    pub fn extend_inherited(&mut self, idents: IndexSet<String>) {
        self.inherited.extend(idents)
    }

    /// get the union of the inherited and introduced identifiers, consuming self
    pub fn into_union(self) -> IndexSet<String> {
        let mut inherited = self.inherited;
        let introduced = self.introduced;
        inherited.extend(introduced);
        inherited
    }
}

/// The actor of the ScopeBuilder pass, this is the result of calling [`Module::annotate_scopes`]
///
/// You may query a scope-node (a node that is considered a scope)
/// for all identifiers it has in scope with [`ScopeAnnotations::get_scope_for`]
///
/// note that [`ScopeAnnotations::get_scope_for`]
/// does not verify that the given id is of a scope-node.
pub struct ScopeAnnotations {
    // Map some node id to all the identifiers the node has in scope
    scopes: IndexMap<usize, NodeScope>,
}
impl ScopeAnnotations {
    pub fn new() -> Self {
        Self {
            scopes: IndexMap::new(),
        }
    }

    pub fn extend_with_introduced<'iter>(&mut self, id: usize, idents: IndexSet<String>) {
        self.scopes.entry(id).or_default().extend_introduced(idents)
    }

    pub fn append_with_introduced<'iter>(&mut self, id: usize, ident: &str) {
        self.scopes.entry(id).or_default().append_introduced(ident);
    }

    pub fn get_inherited_identifiers_of(&mut self, id: usize) -> &IndexSet<String> {
        &self.scopes.entry(id).or_default().inherited
    }

    pub fn get_introduced_identifiers_of(&mut self, id: usize) -> &IndexSet<String> {
        &self.scopes.entry(id).or_default().introduced
    }

    pub fn get_all_identifiers_of(&mut self, id: usize) -> IndexSet<String> {
        self.scopes.entry(id).or_default().clone().into_union()
    }

    pub fn get_scope_for(&mut self, id: usize) -> &NodeScope {
        self.scopes.entry(id).or_default()
    }

    pub fn child_inherits_from(&mut self, parent_id: usize, child_id: usize) {
        let parent_idents = self.get_all_identifiers_of(parent_id).clone();
        self.scopes
            .insert(child_id, NodeScope::from_inherited(parent_idents));
    }

    pub fn inherited_scope_contains(&mut self, scope_id: usize, ident: &str) -> bool {
        self.get_inherited_identifiers_of(scope_id).contains(ident)
    }

    pub fn introduced_scope_contains(&mut self, scope_id: usize, ident: &str) -> bool {
        self.get_introduced_identifiers_of(scope_id).contains(ident)
    }

    pub fn is_shadowed_in(&mut self, scope_id: usize, ident: &str) -> bool {
        self.introduced_scope_contains(scope_id, ident)
            && self.inherited_scope_contains(scope_id, ident)
    }
}

pub trait AcceptScopeBuilder {
    ///
    ///
    /// # Arguments
    ///
    /// * `scope_builder`: the scope builder
    /// * `nearest_scope_id`: the id of the nearest ancestor
    /// that can have direct children that introduce identifiers,
    /// i.e. (a block, a function body, the branches in an if-expression, a module, etc.)
    ///
    /// Note that if the called node that is itself not a scope,
    /// but contains some data `d` that is a scope,
    /// that `d` should first inherit from `nearest_scope_id`,
    /// then call [`d.build_scope_with_parent(d.id)`]:
    ///
    /// ```
    /// match node.kind {
    ///     SomeNodeKind::VariantThatContainsScope(d) => {
    ///         scope_builder.child_inherits_from(nearest_scope_id, d.id);
    ///         d.build_scope_for(d.id, scope_builder);
    ///     }
    ///     _ => todo!()
    /// }
    /// ```
    ///
    /// As such, any scope-callee of [`AcceptScopeBuilder::build_scope_with_parent`]
    /// is guaranteed to have already inherited all identifiers in scope,
    /// and should only make SCOPED subnodes inherit identifiers that the callee itself introduces.
    ///
    /// Any non-scope-callee of [`AcceptScopeBuilder::build_scope_with_parent`]
    /// is guaranteed to have access to the node-id of nearest scoped ancestor,
    /// and should pass this id to any calls to [`AcceptScopeBuilder::build_scope_with_parent`]
    /// for its children.
    ///
    /// The overall structure of an implementation is as the following pseudocode:
    /// ```
    /// forall AcceptScopeBuilder-scoped-children 'sc' of callee {
    ///     'sc' inherits from 'nearest_scope_id';
    ///     'sc'.build_scope('sc'.id)
    /// }
    /// forall AcceptScopeBuilder-non-scoped children 'nsc' of callee {
    ///     'nsc'.build_scope('nearest_scope_id');
    /// }
    /// ```
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    );
}

impl AcceptScopeBuilder for PartiallyUnpackedInitialisation {
    fn build_scope_with_parent(
        &self,
        nearest_scoped_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        let new_identifiers = self.get_introduced_identifiers();
        scope_builder.extend_with_introduced(nearest_scoped_id, new_identifiers);

        self.temporary
            .build_scope_with_parent(nearest_scoped_id, scope_builder);
        for assignment in self.unpacked_assignments.iter() {
            assignment.build_scope_with_parent(nearest_scoped_id, scope_builder);
        }
    }
}

impl AcceptScopeBuilder for UnpackedInitialisation {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        let new_identifiers = self.get_introduced_identifiers();
        scope_builder.extend_with_introduced(nearest_scope_id, new_identifiers);
        self.value
            .build_scope_with_parent(nearest_scope_id, scope_builder);
    }
}

impl AcceptScopeBuilder for PackedInitialisation {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        let my_assignees = self.get_introduced_identifiers();
        scope_builder.extend_with_introduced(nearest_scope_id, my_assignees);
        self.value
            .build_scope_with_parent(nearest_scope_id, scope_builder);
    }
}

impl AcceptScopeBuilder for Initialization {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        match &self.kind {
            InitialisationKind::Packed(p) => {
                p.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            InitialisationKind::PartiallyUnpacked(p) => {
                p.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
        }
    }
}

impl AcceptScopeBuilder for Reassignment {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        self.value
            .build_scope_with_parent(nearest_scope_id, scope_builder)
    }
}

impl AcceptScopeBuilder for FunctionCall {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        for arg in self.args.iter() {
            arg.build_scope_with_parent(nearest_scope_id, scope_builder);
        }
    }
}

impl AcceptScopeBuilder for IfThenElse {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        self.condition
            .build_scope_with_parent(nearest_scope_id, scope_builder);

        self.true_case
            .build_scope_with_parent(nearest_scope_id, scope_builder);

        if let Some(false_case) = &self.false_case {
            false_case.build_scope_with_parent(nearest_scope_id, scope_builder)
        }
    }
}

impl AcceptScopeBuilder for ExpandedBlockExpr {
    fn build_scope_with_parent(
        &self,
        _nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        for stmt in self.statements.iter() {
            scope_builder.child_inherits_from(self.id, stmt.id);
            stmt.build_scope_with_parent(self.id, scope_builder);
        }

        self.last.build_scope_with_parent(self.id, scope_builder);
    }
}

impl AcceptScopeBuilder for Statement {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        match &self.kind {
            StatementKind::Initialisation(i) => {
                i.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            StatementKind::Reassignment(r) => {
                r.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            StatementKind::FunctionCall(call) => {
                call.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            StatementKind::Return(e) => e.build_scope_with_parent(nearest_scope_id, scope_builder),
            StatementKind::BlockTail(e) => {
                e.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            StatementKind::ExpandedBlock(eb) => {
                scope_builder.child_inherits_from(nearest_scope_id, eb.id);
                eb.build_scope_with_parent(eb.id, scope_builder)
            }
            StatementKind::IfThenElse(branch) => {
                branch.build_scope_with_parent(nearest_scope_id, scope_builder)
            }
            StatementKind::Block(_) => unreachable!(),
        }
    }
}

impl AcceptScopeBuilder for Expression {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        match &self.kind {
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::Ident(_) => {}
            ExpressionKind::FunctionCall(call) => {
                for arg in call.args.iter() {
                    arg.build_scope_with_parent(nearest_scope_id, scope_builder);
                }
            }
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.build_scope_with_parent(nearest_scope_id, scope_builder);
                rhs.build_scope_with_parent(nearest_scope_id, scope_builder);
            }
            ExpressionKind::UnOpExpr(_, arg) => {
                arg.build_scope_with_parent(nearest_scope_id, scope_builder);
            }
            ExpressionKind::MemberAccess(datatype, _) => {
                datatype.build_scope_with_parent(nearest_scope_id, scope_builder);
            }
            ExpressionKind::IfThenElse(b) => {
                b.build_scope_with_parent(nearest_scope_id, scope_builder);
            }
            ExpressionKind::ExpandedBlock(e) => {
                scope_builder.child_inherits_from(nearest_scope_id, e.id);
                e.build_scope_with_parent(nearest_scope_id, scope_builder);
            }
            ExpressionKind::Block(_) => {
                unreachable!("AST should not have un-expanded blocks in scope-builder-pass")
            }
        }
    }
}

impl AcceptScopeBuilder for Function {
    fn build_scope_with_parent(&self, module_id: usize, scope_builder: &mut ScopeAnnotations) {
        scope_builder.child_inherits_from(module_id, self.body.id);
        let locals = self.body.get_introduced_identifiers();
        scope_builder.extend_with_introduced(self.body.id, locals);

        for stmt in self.body.statements.iter() {
            stmt.build_scope_with_parent(self.body.id, scope_builder);
        }
    }
}

impl AcceptScopeBuilder for Module {
    fn build_scope_with_parent(&self, _dummy: usize, scope_builder: &mut ScopeAnnotations) {
        let all_globals = self.get_globally_scoped_identifiers();
        scope_builder.extend_with_introduced(self.id, all_globals);
        for global_var in self.global_vars.iter() {
            global_var.build_scope_with_parent(self.id, scope_builder);
        }

        for func in self.functions.iter() {
            func.build_scope_with_parent(self.id, scope_builder);
        }
    }
}

/// This visitor will be called after each of the expansion-visitors
/// to ensure a correct AST before moving on to static analysis.
pub struct ASTValidator {
    ids: HashSet<usize>,
}
pub enum SemanticASTViolation<'ast> {
    UntypedGlobalVar(&'ast Initialization),
    UnexpandedBlock(&'ast Statement),
    StrayPackedAssignment(&'ast Initialization),
}

macro_rules! visit_annotate {
    (block $ast_l:lifetime $f:expr) => {
        fn visit_block(&mut self, block: &'ast ExpandedBlockExpr) -> Result<(), SemanticASTViolation<$ast_l>> {
            let f: fn(&$ast_l ExpandedBlockExpr) -> Result<(), SemanticASTViolation<$ast_l>> = $f;
            f(block)
        }
    };
    (unexp_block $ast_l:lifetime $f:expr) => {
        fn visit_unexpanded_block(&mut self, block: &'ast StatementBlock) -> Result<(), SemanticASTViolation<$ast_l>> {
            let f: fn(&$ast_l StatementBlock) -> Result<(), SemanticASTViolation<$ast_l>> = $f;
            f(block)
        }
    };
    (module_global_variable $ast_l:lifetime $f:expr) => {
        fn visit_module_global_variable(&mut self, module: &$ast_l Module) -> Result<(), SemanticASTViolation<$ast_l>> {
            let f: fn(&$ast_l Initialization) -> Result<(), SemanticASTViolation<$ast_l>> = $f;
            for var in module.global_vars.iter() {
                f(var)?;
            }
            Ok(())
        }
    };
    (module_function_definition $ast_l:lifetime $f:expr) => {
        fn visit_module_function_definition(&mut self, module: &$ast_l Module) -> Result<(), SemanticASTViolation<$ast_l>> {
            let f: fn(&$ast_l Function) -> Result<(), SemanticASTViolation<$ast_l>> = $f;
            for func in module.functions.iter() {
                f(func)?;
            }
            Ok(())
        }
    };
    (initialization $ast_l:lifetime $f:expr) => {
        fn visit_initialization(&mut self, module: &$ast_l Module) -> Result<(), SemanticASTViolation<$ast_l>> {
            let f: fn(&$ast_l Function) -> Result<(), SemanticASTViolation<$ast_l>> = $f;
            for func in module.functions.iter() {
                f(func)?;
            }
            Ok(())
        }
    };
}
pub(crate) use visit_annotate;

macro_rules! annotating_visitor {
    ($ast_l:lifetime $name:ident with $($impl_item:item)*) => {
        impl<'ast> AnnotatingVisitor<'ast> for $name {
            $($impl_item)*
        }
    };
}

annotating_visitor! {'ast ASTValidator with
    visit_annotate!(block 'ast
        |_block| Ok(())
    );
}
pub(crate) use annotating_visitor;

pub trait AcceptAnnotatingVisitor<'ast> {
    /// Let the visitor traverse all children of this node
    fn visit_children(
        &'ast self,
        _visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    /// Let the visitor perform its annotation in this node
    fn accept(
        &'ast self,
        _visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
}

/// This trait defines what behavior an implementing visitor shows for each type of AST node.
///
/// This is inspired by the Python [Lark](https://lark-parser.readthedocs.io/en/stable/visitors.html)
/// package.
///
/// This is done by using the [`annotating_visitor`] macro
/// and providing a lambda for each AST node that the visitor should do something with:
///
/// # Examples
/// ```
/// pub struct SomeVisitor;
///
/// annotating_visitor!{'ast SomeVisitor
///     visit_annotate!{block 'ast
///         |block| {println!(block.id); Ok(())}
///     }
/// }
///
/// fn do<'ast>(ast: &'ast Module) -> Result<(), SemanticASTViolation<'ast>>{
///     let v = SomeVisitor::new();
///     v.traverse(ast)
/// }
/// ```
pub trait AnnotatingVisitor<'ast>: Sized {
    fn visit_block(
        &mut self,
        _v: &'ast ExpandedBlockExpr,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_unexpanded_block(
        &mut self,
        _v: &'ast StatementBlock,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_stmt(&mut self, _v: &'ast Statement) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_initialization(
        &mut self,
        _v: &'ast Initialization,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_reassignment(
        &mut self,
        _v: &'ast Reassignment,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_if_then_else(
        &mut self,
        _v: &'ast IfThenElse,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_func_call(
        &mut self,
        _v: &'ast FunctionCall,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_return(&mut self, _v: &'ast Expression) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_block_tail(&mut self, _v: &'ast Expression) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_module(&mut self, _v: &'ast Module) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_module_global_var(
        &mut self,
        _id: usize,
        _v: &'ast PackedInitialisation,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_module_struct_definition(
        &mut self,
        _v: &'ast StructDataTypeDefinition,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_module_enum_definition(
        &mut self,
        _v: &'ast TaggedUnionDataTypeDefinition,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_module_function_definition(
        &mut self,
        _v: &'ast Function,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr(&mut self, _v: &'ast Expression) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_binop(
        &mut self,
        _id: usize,
        _op: &'ast BinOp,
        _lhs: &'ast Expression,
        _rhs: &'ast Expression,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_unop(
        &mut self,
        _id: usize,
        _op: &'ast UnOp,
        _arg: &'ast Expression,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_ident(
        &mut self,
        _id: usize,
        _v: &'ast String,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_literal_int(
        &mut self,
        _id: usize,
        _v: &'ast u64,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_literal_float(
        &mut self,
        _id: usize,
        _v: &'ast f64,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_literal_bool(
        &mut self,
        _id: usize,
        _v: &'ast bool,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_literal_string(
        &mut self,
        _id: usize,
        _v: &'ast String,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_unit(&mut self, _id: usize) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_if_then_else(
        &mut self,
        _id: usize,
        _v: &'ast IfThenElse,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_function_call(
        &mut self,
        _id: usize,
        _v: &'ast FunctionCall,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_block(
        &mut self,
        _id: usize,
        _v: &'ast ExpandedBlockExpr,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_unexpanded_block(
        &mut self,
        _id: usize,
        _v: &'ast StatementBlock,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn visit_expr_member_access(
        &mut self,
        _id: usize,
        _data: &'ast Expression,
        _member: &'ast String,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        Ok(())
    }
    fn traverse(&mut self, module: &'ast Module) -> Result<(), SemanticASTViolation<'ast>> {
        module.accept(self)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for Expression {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        match &self.kind {
            ExpressionKind::Unit => visitor.visit_expr_unit(self.id),
            ExpressionKind::IntegerLiteral(v) => visitor.visit_expr_literal_int(self.id, v),
            ExpressionKind::BoolLiteral(v) => visitor.visit_expr_literal_bool(self.id, v),
            ExpressionKind::FloatLiteral(v) => visitor.visit_expr_literal_float(self.id, v),
            ExpressionKind::StringLiteral(v) => visitor.visit_expr_literal_string(self.id, v),
            ExpressionKind::Ident(v) => visitor.visit_expr_ident(self.id, v),

            ExpressionKind::FunctionCall(v) => {
                visitor.visit_expr_function_call(self.id, v)?;
                v.accept(visitor)
            }
            ExpressionKind::BinOpExpr(op, lhs, rhs) => {
                lhs.accept(visitor)?;
                rhs.accept(visitor)?;
                visitor.visit_expr_binop(self.id, op, lhs, rhs)
            }
            ExpressionKind::UnOpExpr(op, arg) => {
                arg.accept(visitor)?;
                visitor.visit_expr_unop(self.id, op, arg)
            }
            ExpressionKind::MemberAccess(data, member) => {
                data.accept(visitor)?;
                visitor.visit_expr_member_access(self.id, data, member)
            }
            ExpressionKind::IfThenElse(ite) => ite.accept(visitor),
            ExpressionKind::Block(unexp) => unexp.accept(visitor),
            ExpressionKind::ExpandedBlock(b) => b.accept(visitor),
        }
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_expr(self)?;
        self.visit_children(visitor)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for Statement {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        match &self.kind {
            StatementKind::Initialisation(v) => v.accept(visitor),
            StatementKind::Reassignment(v) => v.value.accept(visitor),
            StatementKind::FunctionCall(v) => v.accept(visitor),
            StatementKind::Return(v) => v.accept(visitor),
            StatementKind::BlockTail(v) => v.accept(visitor),
            StatementKind::Block(v) => v.accept(visitor),
            StatementKind::ExpandedBlock(v) => v.accept(visitor),
            StatementKind::IfThenElse(v) => v.accept(visitor),
        }
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_stmt(self)?;
        self.visit_children(visitor)
    }
}
impl<'ast> AcceptAnnotatingVisitor<'ast> for Module {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        for init in self.global_vars.iter() {
            init.accept(visitor)?;
        }
        for func in self.functions.iter() {
            for stmt in func.body.statements.iter() {
                stmt.accept(visitor)?;
            }
        }
        Ok(())
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_module(self)?;
        for init in self.global_vars.iter() {
            let InitialisationKind::Packed(ref p) = init.kind else {
                unreachable!(
                    "global initializations should be packed; got {:?}",
                    init.kind
                )
            };
            visitor.visit_module_global_var(init.id, p)?;
        }
        for func in self.functions.iter() {
            visitor.visit_module_function_definition(func)?;
        }
        for data_struct in self.struct_definitions.iter() {
            visitor.visit_module_struct_definition(data_struct)?;
        }

        // for data_enum in self.enum_definitions.iter() {
        //     visitor.visit_module_enum_definition(data_enum)?;
        // }
        self.visit_children(visitor)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for StatementBlock {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        for stmt in self.statements.iter() {
            stmt.accept(visitor)?;
        }
        Ok(())
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_unexpanded_block(self)?;
        self.visit_children(visitor)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for ExpandedBlockExpr {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        for stmt in self.statements.iter() {
            stmt.accept(visitor)?;
        }
        self.last.accept(visitor)
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_block(self)?;
        self.visit_children(visitor)
    }
}
impl<'ast> AcceptAnnotatingVisitor<'ast> for IfThenElse {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        self.condition.accept(visitor)?;
        self.true_case.accept(visitor)?;
        if let Some(ref false_case) = self.false_case {
            false_case.accept(visitor)?;
        }
        Ok(())
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_if_then_else(self)?;
        self.visit_children(visitor)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for FunctionCall {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        for arg in self.args.iter() {
            arg.accept(visitor)?;
        }
        Ok(())
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_func_call(self)?;
        self.visit_children(visitor)
    }
}

impl<'ast> AcceptAnnotatingVisitor<'ast> for Initialization {
    fn visit_children(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        match &self.kind {
            InitialisationKind::Packed(p) => p.value.accept(visitor),
            InitialisationKind::PartiallyUnpacked(u) => {
                for unpack in u.unpacked_assignments.iter() {
                    unpack.accept(visitor)?;
                }
                u.temporary.value.accept(visitor)
            }
        }
    }
    fn accept(
        &'ast self,
        visitor: &mut impl AnnotatingVisitor<'ast>,
    ) -> Result<(), SemanticASTViolation<'ast>> {
        visitor.visit_initialization(self)?;
        self.visit_children(visitor)
    }
}

#[cfg(test)]
mod introduces_fresh_identifiers_tests {
    use super::*;
    use crate::zea::test_ast_macros::{block, expr, func, pat, stmt, ztyp};

    #[test]
    fn test_packed_initialization_introduces_single_ident() {
        let init = Initialization::packed(None, pat!(a), expr!(litint 1));
        let introduced = init.get_introduced_identifiers();
        assert!(introduced.contains("a"), "should introduce 'a'");
        assert_eq!(introduced.len(), 1);
    }

    #[test]
    fn test_packed_initialization_introduces_tuple() {
        let init = Initialization::packed(None, pat!((a, b)), expr!(ident some_tuple));
        let introduced = init.get_introduced_identifiers();
        assert!(introduced.contains("a"), "should introduce 'a'");
        assert!(introduced.contains("b"), "should introduce 'b'");
        assert_eq!(introduced.len(), 2);
    }

    #[test]
    fn test_nested_tuple_pattern() {
        let init = Initialization::packed(None, pat!((a, (b, c))), expr!(ident nested));
        let introduced = init.get_introduced_identifiers();
        assert!(introduced.contains("a"));
        assert!(introduced.contains("b"));
        assert!(introduced.contains("c"));
        assert_eq!(introduced.len(), 3);
    }

    #[test]
    fn test_function_introduces_function_name() {
        use crate::zea::{Function, Statement, StatementBlock, StatementKind, TypeSpecifier};
        let body = block!(stmt!(init pat!(a) ;= expr!(litint 3)));
        let func = func!(foo() -> ztyp!(Int); {body} );
        let introduced = func.get_introduced_identifiers();
        assert!(
            introduced.contains("foo"),
            "function should introduce its own name"
        );
    }
}

#[cfg(test)]
mod scope_builder_tests {
    use super::*;
    use crate::zea::test_ast_macros::{block, expr, func, pat, stmt, ztyp};
    use crate::zea::BareNodeLabeler;

    #[test]
    fn test_global_var_in_module_scope() {
        use crate::zea::{Initialization, Module};

        let global_init = Initialization::packed(None, pat!(global_var), expr!(litint 1));
        let mut module = Module {
            id: 1,
            imports: vec![],
            exports: vec![],
            global_vars: vec![global_init],
            functions: vec![],
            struct_definitions: vec![],
        };
        let (module, _) = module.give_ids(BareNodeLabeler::new());

        let mut scopes = module.annotate_scopes();
        let global_scope = scopes.get_introduced_identifiers_of(module.id);
        assert!(
            global_scope.contains("global_var"),
            "global var should be in module scope"
        );
    }

    #[test]
    fn test_function_body_scope_has_locals() {
        use crate::zea::{
            Function, Initialization, Module, Statement, StatementBlock, StatementKind,
            TypeSpecifier,
        };

        let body = block!(stmt!(init  pat!(local_var) ;= expr!(litint 1)));
        let func = func!(foo() -> ztyp!(Int); {body});
        let mut module = Module {
            id: 1,
            imports: vec![],
            exports: vec![],
            global_vars: vec![],
            functions: vec![func],
            struct_definitions: vec![],
        };
        let (module, _) = module.give_ids(BareNodeLabeler::new());

        let mut scopes = module.annotate_scopes();

        let func = &module.functions[0];
        let body_scope = scopes.get_all_identifiers_of(func.body.id);
        assert!(
            body_scope.contains("local_var"),
            "local var should be in function body scope"
        );
    }
    #[test]
    fn test_function_body_scope_has_locals_and_globals() {
        use crate::zea::{
            Function, Initialization, Module, Statement, StatementBlock, StatementKind,
            TypeSpecifier,
        };
        let global = Initialization::packed(None, pat!(global_var), expr!(litint 1));
        let body = block!(stmt!(init  pat!(local_var) ;= expr!(litint 1)));
        let func = func!(foo() -> ztyp!(Int); {body});
        let mut module = Module {
            id: 1,
            imports: vec![],
            exports: vec![],
            global_vars: vec![global],
            functions: vec![func],
            struct_definitions: vec![],
        };
        let (module, _) = module.give_ids(BareNodeLabeler::new());

        let mut scopes = module.annotate_scopes();

        let func = &module.functions[0];
        let body_scope = dbg!(scopes.get_all_identifiers_of(func.body.id));
        assert!(
            body_scope.contains("local_var"),
            "local var should be in function body scope"
        );
        assert!(
            body_scope.contains("global_var"),
            "global var should be in function body scope"
        );
        let expected = IndexSet::from(["global_var".to_string(), "local_var".to_string(), "foo".to_string()]);
        assert!(
            body_scope
                .difference(&expected)
                .collect::<IndexSet<&String>>()
                .is_empty(),
            "scope should contain global_var, local_var and foo"
        );
    }

    #[test]
    fn test_scope_for_known_id() {
        let mut scopes = ScopeAnnotations::new();
        scopes.extend_with_introduced(42, IndexSet::from(["x".to_string()]));
        let scope = scopes.get_scope_for(42);
        assert!(scope.introduced.contains("x"));
    }
}
