use crate::helper_impls::StructuralEq;
use crate::zea::visitors::altering::AcceptsBlockExpander;
use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, Initialization, InitializationKind, Module, PackedInitialization,
    PartiallyUnpackedInitialization, Reassignment, SimpleInitialization, Statement, StatementBlock,
    StatementKind,
};
use indexmap::{IndexMap, IndexSet};
use std::collections::HashSet;
use zea_macros::ASTStructuralEq;

pub trait IntroducesFreshIdentifiers {
    /// Get all identifiers introduced in the current scope
    ///
    /// Note that this should return only identifiers that are newly introduced in the current scope,
    /// including shadowed ones.
    ///
    /// In general, the only node that introduces identifiers is an initialization,
    /// and a function definition.
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier>;
}
impl IntroducesFreshIdentifiers for Initialization {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        ScopedIdentifier::scope_locals_with(self.get_assignee_strings(), self.id)
    }
}

impl AssignmentPattern {
    fn get_assignee_strings(&self) -> IndexSet<String> {
        match self {
            AssignmentPattern::Identifier(i) => IndexSet::from([i.clone()]),
            AssignmentPattern::Tuple(v) => {
                v.iter().flat_map(|i| i.get_assignee_strings()).collect()
            }
        }
    }
}

impl SimpleInitialization {
    fn get_assignee_strings(&self) -> IndexSet<String> {
        IndexSet::from([self.assignee.clone()])
    }
}
impl PartiallyUnpackedInitialization {
    fn get_assignee_strings(&self) -> IndexSet<String> {
        let mut temp_id = self.temporary.get_assignee_strings();
        let unpacked_ids: IndexSet<String> = self
            .unpacked_assignments
            .iter()
            .flat_map(|init| init.get_assignee_strings())
            .collect();
        temp_id.extend(unpacked_ids);
        temp_id
    }
}
impl Initialization {
    fn get_assignee_strings(&self) -> IndexSet<String> {
        match &self.kind {
            InitializationKind::Packed(_) | InitializationKind::PartiallyUnpacked(_) => {
                panic!("cannot only build scope for unpacked assignenmts")
            }
            InitializationKind::Unpacked(pu) => pu
                .iter()
                .flat_map(|init| init.get_assignee_strings())
                .collect(),
        }
    }
}
impl IntroducesFreshIdentifiers for Statement {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        match &self.kind {
            StatementKind::Initialization(i) => i.get_introduced_identifiers(),
            _ => IndexSet::default(),
        }
    }
}

impl IntroducesFreshIdentifiers for StatementBlock {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        self.statements
            .iter()
            .flat_map(|stmt| stmt.get_introduced_identifiers())
            .collect()
    }
}

impl IntroducesFreshIdentifiers for ExpandedBlockExpr {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        self.statements
            .iter()
            .flat_map(|stmt| stmt.get_introduced_identifiers())
            .collect()
        // the only way self.last can introduce a new ident
        // is through a block which is a deeper scope; not in this scope.
    }
}

impl IntroducesFreshIdentifiers for Function {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        let mut name: IndexSet<ScopedIdentifier> =
            IndexSet::from([ScopedIdentifier::func_name(self.id, &self.name)]);
        let params: IndexSet<ScopedIdentifier> = self
            .params
            .iter()
            .map(|param| ScopedIdentifier::func_param(param.id, &param.name))
            .collect();
        name.extend(params);
        name
    }
}
impl IntroducesFreshIdentifiers for Module {
    fn get_introduced_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        self.get_globally_scoped_identifiers()
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum ScopedIdentifierKind {
    LocalVar,
    GlobalVar,
    FunctionName,
    FunctionParam,
    ImportItem,
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ScopedIdentifier {
    ident: String,
    origin: usize,
    kind: ScopedIdentifierKind,
}
impl StructuralEq for ScopedIdentifier {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.ident == other.ident && self.kind == other.kind
    }
}

impl ScopedIdentifier {
    pub fn local(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::LocalVar,
        }
    }
    pub fn global(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::GlobalVar,
        }
    }
    pub fn func_name(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::FunctionName,
        }
    }
    pub fn func_param(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::FunctionParam,
        }
    }

    pub fn import_item(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::ImportItem,
        }
    }
    pub fn scope_locals_with(
        idents: IndexSet<String>,
        origin: usize,
    ) -> IndexSet<ScopedIdentifier> {
        idents
            .into_iter()
            .map(|ident| Self {
                ident,
                origin,
                kind: ScopedIdentifierKind::LocalVar,
            })
            .collect()
    }

    pub fn scope_globals_with(
        idents: IndexSet<String>,
        origin: usize,
    ) -> IndexSet<ScopedIdentifier> {
        idents
            .into_iter()
            .map(|ident| Self {
                ident,
                origin,
                kind: ScopedIdentifierKind::GlobalVar,
            })
            .collect()
    }
}

#[derive(Default, Clone)]
/// A scope that contains both inherited and introduced identifiers
#[derive(Debug, ASTStructuralEq)]
pub struct NodeScope {
    inherited: IndexSet<ScopedIdentifier>,
    introduced: IndexSet<ScopedIdentifier>,
}

impl NodeScope {
    pub fn from_inherited(inherited: IndexSet<ScopedIdentifier>) -> Self {
        Self {
            inherited,
            introduced: IndexSet::new(),
        }
    }

    pub fn inherit_from(&mut self, scope: NodeScope) {
        self.inherited.extend(scope.into_union());
    }

    pub fn from_introduced(introduced: IndexSet<ScopedIdentifier>) -> Self {
        Self {
            introduced,
            inherited: IndexSet::new(),
        }
    }
    pub fn append_introduced(&mut self, ident: &ScopedIdentifier) {
        self.introduced.insert(ident.to_owned());
    }
    pub fn extend_introduced(&mut self, idents: IndexSet<ScopedIdentifier>) {
        self.introduced.extend(idents)
    }
    pub fn append_inherited(&mut self, ident: &ScopedIdentifier) {
        self.inherited.insert(ident.to_owned());
    }
    /// extend inherited identifiers with
    pub fn extend_inherited(&mut self, idents: IndexSet<ScopedIdentifier>) {
        self.inherited.extend(idents)
    }

    /// get the union of the inherited and introduced identifiers, consuming self
    pub fn into_union(self) -> IndexSet<ScopedIdentifier> {
        let mut inherited = self.inherited;
        let introduced = self.introduced;
        inherited.extend(introduced);
        inherited
    }

    pub fn inherited_contains_local_str(&self, ident: &str) -> bool {
        self.inherited
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::LocalVar && si.ident == ident)
            .is_some()
    }
    pub fn inherited_contains_global_str(&self, ident: &str) -> bool {
        self.inherited
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::GlobalVar && si.ident == ident)
            .is_some()
    }
    pub fn inherited_contains_func_name_str(&self, ident: &str) -> bool {
        self.inherited
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::FunctionName && si.ident == ident)
            .is_some()
    }
    pub fn inherited_contains_func_param_str(&self, ident: &str) -> bool {
        self.inherited
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::FunctionParam && si.ident == ident)
            .is_some()
    }
    pub fn inherited_contains_import_item_str(&self, ident: &str) -> bool {
        self.inherited
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::ImportItem && si.ident == ident)
            .is_some()
    }

    pub fn introduced_contains_local_str(&self, ident: &str) -> bool {
        self.introduced
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::LocalVar && si.ident == ident)
            .is_some()
    }
    pub fn introduced_contains_global_str(&self, ident: &str) -> bool {
        self.introduced
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::GlobalVar && si.ident == ident)
            .is_some()
    }
    pub fn introduced_contains_func_name_str(&self, ident: &str) -> bool {
        self.introduced
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::FunctionName && si.ident == ident)
            .is_some()
    }
    pub fn introduced_contains_func_param_str(&self, ident: &str) -> bool {
        self.introduced
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::FunctionParam && si.ident == ident)
            .is_some()
    }
    pub fn introduced_contains_import_item_str(&self, ident: &str) -> bool {
        self.introduced
            .iter()
            .find(|si| si.kind == ScopedIdentifierKind::ImportItem && si.ident == ident)
            .is_some()
    }

    pub fn contains_local_str(&self, ident: &str) -> bool {
        self.introduced_contains_local_str(ident) || self.inherited_contains_local_str(ident)
    }
    pub fn contains_global_str(&self, ident: &str) -> bool {
        self.inherited_contains_local_str(ident) || self.introduced_contains_global_str(ident)
    }
    pub fn contains_func_name_str(&self, ident: &str) -> bool {
        self.inherited_contains_func_name_str(ident)
            || self.introduced_contains_func_name_str(ident)
    }
    pub fn contains_func_param_str(&self, ident: &str) -> bool {
        self.introduced_contains_func_param_str(ident)
            || self.inherited_contains_func_param_str(ident)
    }
    pub fn contains_import_item_str(&self, ident: &str) -> bool {
        self.inherited_contains_import_item_str(ident)
            || self.introduced_contains_import_item_str(ident)
    }
}

/// The actor of the ScopeBuilder pass, this is the result of calling [`Module::annotate_scopes`]
///
/// You may query a scope-node (a node that is considered a scope)
/// for all identifiers it has in scope with [`ScopeAnnotations::get_scope`]
///
/// note that [`ScopeAnnotations::get_scope`]
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

    pub fn extend_with_introduced<'iter>(&mut self, id: usize, idents: IndexSet<ScopedIdentifier>) {
        self.scopes.entry(id).or_default().extend_introduced(idents)
    }

    pub fn append_with_introduced<'iter>(&mut self, id: usize, ident: &ScopedIdentifier) {
        self.scopes.entry(id).or_default().append_introduced(ident);
    }

    pub fn get_inherited_identifiers_of(&mut self, id: usize) -> &IndexSet<ScopedIdentifier> {
        &self.scopes.entry(id).or_default().inherited
    }

    pub fn get_introduced_identifiers_of(&mut self, id: usize) -> &IndexSet<ScopedIdentifier> {
        &self.scopes.entry(id).or_default().introduced
    }

    pub fn get_all_identifiers_of(&mut self, id: usize) -> IndexSet<ScopedIdentifier> {
        self.scopes.entry(id).or_default().clone().into_union()
    }

    pub fn get_scope(&mut self, id: usize) -> &NodeScope {
        self.scopes.entry(id).or_default()
    }

    pub fn child_inherits_from(&mut self, parent_id: usize, child_id: usize) {
        let parent_idents = self.get_all_identifiers_of(parent_id).clone();
        self.scopes
            .insert(child_id, NodeScope::from_inherited(parent_idents));
    }

    pub fn inherited_scope_contains(&mut self, scope_id: usize, ident: &ScopedIdentifier) -> bool {
        self.get_inherited_identifiers_of(scope_id).contains(ident)
    }

    pub fn introduced_scope_contains(&mut self, scope_id: usize, ident: &ScopedIdentifier) -> bool {
        self.get_introduced_identifiers_of(scope_id).contains(ident)
    }

    // pub fn is_shadowed_in(&mut self, scope_id: usize, ident: &ScopedIdentifier) -> bool {
    //     self.introduced_scope_contains(scope_id, ident)
    //         && self.inherited_scope_contains(scope_id, ident)
    // }
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

    fn get_scope_annotations(&self) -> ScopeAnnotations {
        let mut new = ScopeAnnotations::new();
        self.build_scope_with_parent(0, &mut new);
        new
    }
}
impl AcceptScopeBuilder for SimpleInitialization {
    fn build_scope_with_parent(
        &self,
        nearest_scope_id: usize,
        scope_builder: &mut ScopeAnnotations,
    ) {
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
            InitializationKind::PartiallyUnpacked(_) | InitializationKind::Packed(_) => {
                panic!("can only build scope for unpacked initialization")
            }
            InitializationKind::Unpacked(v) => {
                let new = self.get_introduced_identifiers();
                scope_builder.extend_with_introduced(nearest_scope_id, new);
                for init in v.iter() {
                    init.build_scope_with_parent(nearest_scope_id, scope_builder)
                }
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
            StatementKind::Initialization(i) => {
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
        let mut global_vars = IndexSet::new();
        for glob in self.global_vars.iter() {
            let assignees = glob.get_assignee_strings();
            global_vars.extend(ScopedIdentifier::scope_globals_with(assignees, glob.id));
        }

        scope_builder.extend_with_introduced(self.id, global_vars);
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
    StrayPackedAssignment(&'ast PackedInitialization),
}

#[cfg(test)]
mod introduces_fresh_identifiers_tests {
    use super::*;
    use crate::zea::test_ast_macros::{block, expr, func, pat, stmt, ztyp};
    use crate::zea::visitors::label_desugar;

    #[test]
    fn test_packed_initialization_introduces_single_ident() {
        let init = Initialization::packed(None, pat!(a), expr!(litint 1));
        let (init, _) = label_desugar(init);
        let introduced = dbg!(NodeScope::from_introduced(
            init.get_introduced_identifiers()
        ));
        assert!(introduced.contains_local_str("a"), "should introduce 'a'");
        assert_eq!(introduced.introduced.len(), 1);
    }

    #[test]
    fn test_packed_initialization_introduces_tuple() {
        let init = Initialization::packed(None, pat!((a, b)), expr!(ident some_tuple));
        let (init, _) = label_desugar(init);
        let mut introduced = init.get_scope_annotations();
        let introduced = introduced.get_scope(init.id);
        assert!(introduced.contains_local_str("a"), "should introduce 'a'");
        assert!(introduced.contains_local_str("b"), "should introduce 'b'");
        assert_eq!(introduced.introduced.len(), 2);
    }

    #[test]
    fn test_nested_tuple_pattern() {
        let init = Initialization::packed(None, pat!((a, (b, c))), expr!(ident nested));
        let (init, _) = label_desugar(init);

        let introduced = NodeScope::from_introduced(init.get_introduced_identifiers());
        assert!(introduced.contains_local_str("a"));
        assert!(introduced.contains_local_str("b"));
        assert!(introduced.contains_local_str("c"));
    }

    #[test]
    fn test_function_introduces_function_name() {
        use crate::zea::{Function, Statement, StatementBlock, StatementKind, TypeSpecifier};
        let body = block!(stmt!(init pat!(a) ;= expr!(litint 3)));
        let func = func!(foo() -> ztyp!(Int); {body} );
        let (func, _) = label_desugar(func);
        let introduced = NodeScope::from_introduced(func.get_introduced_identifiers());
        assert!(
            introduced.contains_func_name_str("foo"),
            "function should introduce its own name"
        );
    }
}

#[cfg(test)]
mod scope_builder_tests {
    use super::*;
    use crate::zea::test_ast_macros::{block, expr, func, pat, stmt, zea_module, ztyp};
    use crate::zea::visitors::altering::LabelSentinelIDs;
    use crate::zea::visitors::label_desugar;
    use crate::zea::{
        Function, Initialization, Module, Statement, StatementBlock, StatementKind, TypeSpecifier,
    };

    #[test]
    fn test_global_var_in_module_scope() {
        let global_init = Initialization::packed(None, pat!(global_var), expr!(litint 1));
        let (module, _g) = label_desugar(zea_module!(
            imports {}
            exports {}
            globs { global_init }
            funcs {}
            structs {}
        ));

        let mut scopes = module.annotate_scopes();
        let global_scope = dbg!(scopes.get_scope(module.id));
        assert!(
            global_scope.contains_global_str("global_var"),
            "global var should be in module scope"
        );
    }

    #[test]
    fn test_function_body_scope_has_locals() {
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
        module.label_sentinel_ids();

        let mut scopes = module.get_scope_annotations();

        let func = &module.functions[0];
        let body_scope = scopes.get_scope(func.body.id);
        assert!(
            body_scope.contains_local_str("local_var"),
            "local var should be in function body scope"
        );
    }
    #[test]
    fn test_function_body_scope_has_locals_and_globals() {
        let global = Initialization::packed(None, pat!(global_var), expr!(litint 1));
        let body = block!(stmt!(init  pat!(local_var) ;= expr!(litint 1)));
        let func = func!(foo() -> ztyp!(Int); {body});
        let module = Module {
            id: 1,
            imports: vec![],
            exports: vec![],
            global_vars: vec![global],
            functions: vec![func],
            struct_definitions: vec![],
        };
        let (module, _) = label_desugar(module);

        let mut scopes = module.annotate_scopes();

        let func = &module.functions[0];
        let body_scope = dbg!(scopes.get_scope(func.body.id));
        assert!(
            body_scope.contains_local_str("local_var"),
            "local var should be in function body scope"
        );
        assert!(
            body_scope.contains_global_str("global_var"),
            "global var should be in function body scope"
        );

        assert!(
            body_scope.contains_func_name_str("foo"),
            "func foo should be in function body scope"
        );
    }

    #[test]
    fn test_scope_for_known_id() {
        let mut scopes = ScopeAnnotations::new();
        scopes.extend_with_introduced(42, IndexSet::from([ScopedIdentifier::local(41, "x")]));
        let scope = scopes.get_scope(42);
        assert!(scope.introduced.contains(&ScopedIdentifier::local(41, "x")));
    }
}
