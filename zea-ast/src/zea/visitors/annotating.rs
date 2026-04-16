use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, Initialisation, InitialisationKind, Module, PackedInitialisation,
    PartiallyUnpackedInitialisation, Reassignment, Statement, StatementBlock, StatementKind,
    UnpackedInitialisation,
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
impl IntroducesFreshIdentifiers for Initialisation {
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
        let parent_idents = self.get_inherited_identifiers_of(parent_id).clone();
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

impl AcceptScopeBuilder for Initialisation {
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
            StatementKind::CondBranch(branch) => {
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
            ExpressionKind::FuncCall(call) => {
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
            ExpressionKind::CondBranch(b) => {
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
    fn build_scope_with_parent(
        &self,
        _nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
        let locals = self.body.get_introduced_identifiers();
        scope_builder.extend_with_introduced(self.body.id, locals);

        for stmt in self.body.statements.iter() {
            stmt.build_scope_with_parent(self.body.id, scope_builder);
        }
    }
}

impl AcceptScopeBuilder for Module {
    fn build_scope_with_parent(
        &self,
        _nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
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
pub enum ASTSemanticViolation<'ast> {
    UntypedGlobalVar(&'ast Initialisation),
    UnexpandedBlock(&'ast Statement),
    StrayPackedAssignment(&'ast Initialisation),
}
pub trait AcceptsAstValidator<'ast> {
    /// Returns true if this node is considered valid
    fn global_vars_are_typed_explicitly(
        &'ast self,
        validator: &mut ASTValidator,
    ) -> Result<(), ASTSemanticViolation<'ast>>;
    fn blocks_are_expanded(
        &'ast self,
        astvalidator: &mut ASTValidator,
    ) -> Result<(), ASTSemanticViolation<'ast>>;
    fn assignments_are_simplified(
        &'ast self,
        astvalidator: &mut ASTValidator,
    ) -> Result<(), ASTSemanticViolation<'ast>>;

    fn tuples_are_named(
        &'ast self,
        astvalidator: &mut ASTValidator,
    ) -> Result<(), ASTSemanticViolation<'ast>>;
}

#[cfg(test)]
mod scope_builder_tests {
    use crate::zea::visitors::annotating::ScopeAnnotations;

    #[test]
    fn block_scopes() {
        let _scope_builder = ScopeAnnotations::new();
    }
}
