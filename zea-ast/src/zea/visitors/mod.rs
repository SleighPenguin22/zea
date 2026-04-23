use indexmap::IndexSet;

pub mod altering;
use crate::zea::visitors::altering::{
    AcceptsAssignmentSimplifier, BlockExpander, LabelSentinelIDs, NodeLabeler,
};
use crate::zea::visitors::annotating::{
    AcceptScopeBuilder, IntroducesFreshIdentifiers, ScopeAnnotations, ScopedIdentifier,
};
use crate::zea::{
    ExpandedBlockExpr, Expression, Function, FunctionCall, IfThenElse, Initialization, Module,
    Reassignment, Statement,
};
use altering::AcceptsBlockExpander;

pub mod annotating;

impl Module {
    pub fn get_globally_scoped_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        let mut global_idents: IndexSet<ScopedIdentifier> = self
            .global_vars
            .iter()
            .flat_map(IntroducesFreshIdentifiers::get_introduced_identifiers)
            .collect();
        let func_idents: IndexSet<ScopedIdentifier> = self
            .functions
            .iter()
            .flat_map(IntroducesFreshIdentifiers::get_introduced_identifiers)
            .collect();
        let import_idents: IndexSet<ScopedIdentifier> = self
            .imports
            .iter()
            .map(|imp| ScopedIdentifier::import_item(self.id, imp))
            .collect();
        global_idents.extend(func_idents);
        global_idents.extend(import_idents);
        global_idents
    }
    pub fn annotate_scopes(&self) -> ScopeAnnotations {
        let mut scope_builder = ScopeAnnotations::new();
        self.build_scope_with_parent(self.id, &mut scope_builder);
        scope_builder
    }
}

pub trait AstNode:
    AcceptsAssignmentSimplifier + AcceptsBlockExpander + AcceptScopeBuilder + LabelSentinelIDs
{
}
impl AstNode for Module {}
impl AstNode for Expression {}
impl AstNode for Statement {}
impl AstNode for IfThenElse {}
impl AstNode for ExpandedBlockExpr {}
impl AstNode for FunctionCall {}
impl AstNode for Function {}

impl AstNode for Initialization {}
impl AstNode for Reassignment {}

pub fn desugar<Node: AstNode>(ast: Node) -> (Node, impl NodeLabeler) {
    let mut ast = ast;
    let generator = ast.expand_blocks();

    let generator = ast.simplify_assignments_with(generator);
    (ast, generator)
}
pub fn desugar_with<Node: AstNode>(
    ast: Node,
    last_used_generator: impl NodeLabeler,
) -> (Node, impl NodeLabeler) {
    let mut ast = ast;
    let generator = ast.expand_blocks_with(last_used_generator);

    let generator = ast.simplify_assignments_with(generator);
    (ast, generator)
}

pub fn label_desugar<Node: AstNode>(ast: Node) -> (Node, impl NodeLabeler) {
    let mut ast = ast;
    let generator = ast.label_sentinel_ids();
    let generator = ast.expand_blocks_with(generator);
    let generator = ast.simplify_assignments_with(generator);
    (ast, generator)
}
pub fn label_desugar_with<Node: AstNode>(
    ast: Node,
    last_used_generator: impl NodeLabeler,
) -> (Node, impl NodeLabeler) {
    let mut ast = ast;
    let generator = ast.label_sentinel_ids_with(last_used_generator);
    let generator = ast.expand_blocks_with(generator);
    let generator = ast.simplify_assignments_with(generator);
    (ast, generator)
}
