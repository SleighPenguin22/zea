pub mod altering;

use crate::zea::visitors::altering::{
    AcceptsAssignmentSimplifier, BlockExpander, IdentifierScope, LabelSentinelIDs, NodeLabeler,
    NotInScopeError,
};
use crate::zea::visitors::annotating::{ScopeAnnotations, ScopedIdentifier};
use crate::zea::{
    ExpandedBlockExpr, Expression, Function, FunctionCall, IfThenElse, Initialization, Module,
    Reassignment, Statement,
};
use altering::AcceptsBlockExpander;
use indexmap::IndexSet;

pub mod annotating;

impl Module {
    pub fn get_globally_scoped_identifiers(&self) -> IndexSet<ScopedIdentifier> {
        let mut annotations = ScopeAnnotations::new();
        annotations.gather_idents_module(self);
        annotations.globals()
    }
    pub fn annotate_scopes(&self) -> ScopeAnnotations {
        todo!()
    }

    pub fn scopify_identifiers(&mut self) -> Result<(), NotInScopeError> {
        let mut annotator = IdentifierScope::new(self);
        annotator.visit_module(self)
    }
}

pub trait AstNode: AcceptsAssignmentSimplifier + AcceptsBlockExpander + LabelSentinelIDs {}
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
