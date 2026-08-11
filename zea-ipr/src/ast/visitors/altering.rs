//!
//! This module contains implementations for the following transformers:
//! - [`BareNodeLabeler`]: give each IPR node a unique label
//! - [`AssignmentExpander`]: expand pattern-initializations into a series of simple initializations
//! - [`IdentifierScoper`]: disambguate identifier-expression and annotate them with their binding site
//! - [`InsertImplicitMainReturn`]: insert a return 0 inside the `main` function, if it exists

use crate::ast::visitors::annotating::IPRScopedIdentifier;
use crate::ast::visitors::{
    IPRTransfomer, IPRVisitor, walk_mut_block, walk_mut_branch, walk_mut_call, walk_mut_expr,
    walk_mut_funcdef, walk_mut_initblock, walk_mut_module, walk_mut_reassignment, walk_mut_stmt,
    walk_mut_structdef, walk_mut_unpacked_init,
};
use crate::ast::{NodeId, ipr::*};
use crate::visualisation::IndentPrint;
use crate::{InternTable, ZeaError, impl_nodelabeler};
use indexmap::IndexSet;
use indexmap::set::MutableValues;
use log::trace;
use std::collections::{HashMap, HashSet};
use std::process::exit;
use zea_internal_macros::{InternKey, VariantToStr};

pub trait NodeLabeler: Sized {
    /// Start `Self`'s id-generator with the last id that `other_generator` used,
    /// such that [`Self::next_id`] calls will never produce an ID
    /// equal to any of `other_generator`'s ID's.
    fn labeler_from(other_generator: impl NodeLabeler) -> Self;
    fn labeler_into<V: NodeLabeler>(self) -> V {
        NodeLabeler::labeler_from(self)
    }
    /// All implementors must ensure that any ID generated is not equal to 0,
    /// as this is a sentinel ID used to signify the need for a fresh ID
    fn next_id(&mut self) -> NodeId;
    /// Generate the next label, along with a valid unique identifier
    fn next_label_with_ident_string(&mut self) -> (NodeId, String) {
        let next = self.next_id();
        (next, format!("__synthetic{}", next))
    }
    /// assign a fresh ID only if the current ID is equal to 0.
    fn update_label(&mut self, current_id: &mut NodeId) {
        if *current_id == NodeId(0) {
            *current_id = self.next_id();
        }
    }
}

use zea_common::{CompilerError, CompilerErrorKind, CompilerStage};
fn stray_packed_init() -> CompilerError {
    CompilerError::new(
        CompilerStage::ExpandInit,
        CompilerErrorKind::StrayPackedInit,
    )
}
macro_rules! internal_compiler_error {
    (spi) => {
        unreachable!("{}", stray_packed_init().pretty())
    };
}

pub struct BareNodeLabeler {
    label: u32,
}

impl BareNodeLabeler {
    pub fn new() -> Self {
        Self { label: 1 }
    }
}

impl NodeLabeler for BareNodeLabeler {
    fn next_id(&mut self) -> NodeId {
        let l = NodeId(self.label);
        self.label += 1;
        l
    }
    fn labeler_from(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_id().0,
        }
    }
}

impl IPRTransfomer for BareNodeLabeler {
    type TransformerOk = ();
    type TransformerError = ();
    fn visit_block(
        &mut self,
        block: &mut IPRBlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut block.id);
        walk_mut_block(self, block)
    }
    fn visit_branch(
        &mut self,
        branch: &mut IPRBranch,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut branch.id);
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &mut IPRFunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut call.id);
        walk_mut_call(self, call)
    }
    fn visit_expr(
        &mut self,
        expr: &mut IPRExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut expr.id);
        walk_mut_expr(self, expr)
    }
    fn visit_funcdef(
        &mut self,
        funcdef: &mut IPRFunction,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut funcdef.id);
        walk_mut_funcdef(self, funcdef)
    }
    fn visit_init(
        &mut self,
        init: &mut IPRSimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut init.id);
        walk_mut_unpacked_init(self, init)
    }
    fn visit_initblock(
        &mut self,
        init: &mut IPRInitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut init.id);
        walk_mut_initblock(self, init)
    }
    fn visit_module(
        &mut self,
        module: &mut IPRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut module.id);
        walk_mut_module(self, module)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &mut IPRReassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut reinit.id);
        walk_mut_reassignment(self, reinit)
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut IPRStatement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut stmt.id);
        walk_mut_stmt(self, stmt)
    }
    fn visit_structdef(
        &mut self,
        structdef: &mut IPRStructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut structdef.id);
        walk_mut_structdef(self, structdef)
    }
}

pub struct AssignmentExpander {
    label: u32,
}

crate::impl_nodelabeler!(AssignmentExpander, "unpack");

impl IPRTransfomer for AssignmentExpander {
    type TransformerError = ();
    type TransformerOk = ();
    fn visit_initblock(
        &mut self,
        init: &mut IPRInitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerOk> {
        match init.kind {
            IPRInitializationKind::Packed(_) => {
                init.kind = IPRInitializationKind::Unpacked(self.expand_assignment(init.clone()));
            }
            IPRInitializationKind::Unpacked(_) => {}
        }
        walk_mut_initblock(self, init)
    }
}

impl AssignmentExpander {
    pub fn new() -> Self {
        Self { label: 1 }
    }
    /// synthesize a [`SimpleInitialization`] for use in assignment expansion.
    ///
    /// Also generates an expression referencing that initialization.
    fn synthesize_temporary(
        &mut self,
        value: IPRExpression,
    ) -> (IPRSimpleInitialization, IPRExpression) {
        let (synthetic_init_id, label) = self.next_label_with_ident_string();
        let init = IPRSimpleInitialization::untyped(&label, value).with_id(synthetic_init_id);

        let referrant_expr = IPRExpression::scoped_local(label, synthetic_init_id);
        (init, referrant_expr)
    }
    fn synthesize_unpacking_tuple_item(
        &mut self,
        assignee: IPRAssignmentPattern,
        value: IPRExpression,
        index: usize,
    ) -> IPRPackedInitialization {
        let member_access_id = self.next_id();
        let member_access =
            IPRExpression::member_access(value, format!("_{index}")).with_id(member_access_id);
        IPRPackedInitialization::untyped(assignee, member_access)
    }
    fn expand_assignment(&mut self, init: IPRInitializationBlock) -> Vec<IPRSimpleInitialization> {
        match init.kind {
            IPRInitializationKind::Packed(p) => self.expand_packed_init(p),
            IPRInitializationKind::Unpacked(u) => u,
        }
    }
    fn expand_packed_init(
        &mut self,
        init: IPRPackedInitialization,
    ) -> Vec<IPRSimpleInitialization> {
        match init.assignee {
            IPRAssignmentPattern::Identifier(i) => {
                let simple_init_id = self.next_id();
                let simple =
                    IPRSimpleInitialization::untyped(&i, init.value).with_id(simple_init_id);
                vec![simple]
            }
            IPRAssignmentPattern::Tuple(t) => {
                let (temp, ident_expr) = self.synthesize_temporary(init.value.clone());

                let mut res = vec![temp];
                for (index, assignee) in t.into_iter().enumerate() {
                    let referrant_id = self.next_id();
                    let sub_init = self.synthesize_unpacking_tuple_item(
                        assignee,
                        ident_expr.clone().with_id(referrant_id),
                        index,
                    );

                    let recursive_unpacked = self.expand_packed_init(sub_init);
                    res.extend(recursive_unpacked)
                }
                res
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, InternKey)]
pub struct BlockScopeIndex(u32);

impl BlockScopeIndex {
    pub fn sentinel() -> Self {
        Self(0)
    }
    pub fn is_sentinel(&self) -> bool {
        self.0 == 0
    }
}
/// A lexical scope within a block-like node,
/// keeping track of all identifiers introduced within the scope
#[derive(Eq, Hash, PartialEq, Debug)]
pub struct BlockLikeScope {
    /// The node that created this scope; a link to a [`BlockExpression`]
    origin: NodeId,
    /// The parent scope; a link to the [`BlockScope`] of the nearest enclosing [`BlockExpression`]
    parent: BlockScopeIndex,
    /// All identifiers the current scope introduces, in order.
    /// can be used to check if an identifier has been declared at a point in the block.
    introductions: Vec<IPRScopedIdentifier>,
    children: Vec<BlockScopeIndex>,
    kind: ScopeKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScopeKind {
    Function,
    Block,
    BranchTwig,
    Global,
}

impl BlockLikeScope {
    pub fn from_block(block: &IPRBlockExpression, parent: BlockScopeIndex) -> Self {
        Self {
            origin: block.id,
            parent,
            introductions: vec![],
            children: vec![],
            kind: ScopeKind::Block,
        }
    }
    pub fn add_child(&mut self, idx: BlockScopeIndex) -> &mut Self {
        self.children.push(idx);
        self
    }
    pub fn add_introduction(&mut self, ident: IPRScopedIdentifier) -> &mut Self {
        self.introductions.push(ident);
        self
    }
    pub fn introduces(&self, ident: &str) -> Option<&IPRScopedIdentifier> {
        self.introductions.iter().find(|p| p.ident == ident)
    }
    pub fn is_module_root(&self) -> bool {
        self.parent.is_sentinel() && self.kind == ScopeKind::Global
    }
}

/// this pass builds scope information within the AST,
/// and then replaces all occurences of Expression::UnscopedIdent with an
/// [`IPRScopedIdentifier`]
///
/// These scoped identifiers contain the name, kind and origin of their symbol
/// Important to note is that the origin is the *binding site* of the symbol,
/// which is one of the following:
/// - an [`IPRSimpleInitialization`]
/// - an [`IPRFuncParam`]
/// - an [`IPRFunction`]
///
pub struct IdentifierScoper {
    /// The scope-path to the currently analyzed scope
    scope_stack: Vec<BlockScopeIndex>,
    /// A set of all unique scopes within the program
    scope_arena: InternTable<BlockScopeIndex, BlockLikeScope>,
    /// map an Ident-expression to a BlockScope and the nearest enclosing block.
    node_to_scope: HashMap<NodeId, BlockScopeIndex>,
}

#[derive(Debug)]
pub struct NotInScopeError {
    ident: String,
    scope_stack_top: BlockScopeIndex,
}

impl<'m> ZeaError<'m> for NotInScopeError {
    type ErrContext = (IdentifierScoper, IPRModule);
    fn zea_error_format(&self, ctx: &Self::ErrContext) -> String {
        let (scope_ctx, _module) = ctx;
        let origin = scope_ctx.get_scope(self.scope_stack_top);
        let ident = &self.ident;
        let pretty_scope_kind = scopekind_to_pretty_string(origin.kind);
        let cur_scope = scope_ctx.current_scope().kind;
        let in_scope = scope_ctx
            .all_in_current_scope()
            .into_iter()
            .map(|i| &i.ident);
        let mut buffer = format!(
            "(in {cur_scope:?}): identifier `{ident}` not found within {pretty_scope_kind}, identifiers in scope:\n"
        );

        for i in in_scope {
            buffer += &format!("- {i}\n");
        }
        buffer
    }
}

fn scopekind_to_pretty_string(scopekind: ScopeKind) -> &'static str {
    match scopekind {
        ScopeKind::Function => "this function",
        ScopeKind::Block => "this block",
        ScopeKind::BranchTwig => "this branch",
        ScopeKind::Global => "the global scope",
    }
}

pub fn scope_module(mut module: IPRModule) -> IPRModule {
    let mut scoper = IdentifierScoper::new(&module);
    match scoper.visit_module(&mut module) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.zea_error_format(&(scoper, module)));
            exit(1)
        }
    }
    module
}

impl IPRTransfomer for IdentifierScoper {
    type TransformerError = NotInScopeError;
    type TransformerOk = ();
    fn visit_expr(
        &mut self,
        expr: &mut IPRExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        if let IPRExpressionKind::UnScopedIdent(i) = &mut expr.kind {
            trace!("resolving identifier: {i}");
            if i == "true" {
                expr.kind = IPRExpressionKind::BoolLiteral(true);
            } else if i == "false" {
                expr.kind = IPRExpressionKind::BoolLiteral(false);
            } else {
                let scoped_ident = self.resolve(i)?.clone();
                expr.kind = IPRExpressionKind::ScopedIdent(scoped_ident);
            }
            return Ok(());
        }
        walk_mut_expr(self, expr)?;
        Ok(())
    }

    fn visit_init(
        &mut self,
        init: &mut IPRSimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        let ident = IPRScopedIdentifier::local(init.id, init.assignee.clone());
        let cs = self.current_scope_mut();
        cs.add_introduction(ident);
        walk_mut_unpacked_init(self, init)?;
        Ok(())
    }
    fn visit_block(
        &mut self,
        block: &mut IPRBlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.enter_scope(block.id, ScopeKind::Block);
        walk_mut_block(self, block)?;
        self.exit_scope();
        Ok(())
    }
    fn visit_module(
        &mut self,
        module: &mut IPRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        // global variables are considered ordered; their scope covers everything below them.
        // global variables may not be initialized with function calls; they should be constants.
        for glob in module.global_vars.iter_mut() {
            let IPRInitializationKind::Unpacked(u) = &mut glob.kind else {
                internal_compiler_error!(spi)
            };
            trace!("scoping global inits {u:?}");
            for init in u {
                walk_mut_unpacked_init(self, init)?;
                self.current_scope_mut()
                    .add_introduction(IPRScopedIdentifier::from_global_init(init));
            }
        }
        // Functions and imports are not considered ordered; their scope covers the whole of the module.
        for import in module.imports.iter_mut() {
            self.current_scope_mut()
                .add_introduction(IPRScopedIdentifier::import_item(module.id, import.clone()));
        }

        for func in module.functions.iter_mut() {
            self.current_scope_mut()
                .add_introduction(IPRScopedIdentifier::from_funcdef(func));
        }

        for func in module.functions.iter_mut() {
            self.visit_funcdef(func)?;
        }

        Ok(())
    }
    fn visit_funcdef(
        &mut self,
        funcdef: &mut IPRFunction,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        let scope = self.enter_scope(funcdef.body.id, ScopeKind::Function);
        trace!("entering func {}", funcdef.name);
        for param in funcdef.params.iter() {
            scope.add_introduction(IPRScopedIdentifier::from_func_param(param));
        }
        walk_mut_block(self, &mut funcdef.body)?;
        self.exit_scope();
        Ok(())
    }
    fn visit_branch(
        &mut self,
        branch: &mut IPRBranch,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.visit_expr(branch.condition.as_mut())?;

        self.visit_branch_twig(branch.true_case.as_mut())?;

        if let Some(false_case) = branch.false_case.as_mut() {
            self.visit_branch_twig(false_case.as_mut())?;
        }
        Ok(())
    }
}

fn branch_twig_as_block(twig: &mut IPRExpression) -> Option<&mut IPRBlockExpression> {
    match &mut twig.kind {
        IPRExpressionKind::Block(b) => Some(b.as_mut()),
        _ => None,
    }
}

impl IdentifierScoper {
    pub fn new(module: &IPRModule) -> Self {
        let mut new = Self {
            scope_stack: Vec::with_capacity(16),
            scope_arena: InternTable::new(),
            node_to_scope: HashMap::with_capacity(128),
        };
        new.build_global_scope_skeleton(module);
        new
    }
    fn build_global_scope_skeleton(&mut self, module: &IPRModule) -> &mut BlockLikeScope {
        let dummy_scope = BlockLikeScope {
            origin: NodeId::sentinel(),
            parent: BlockScopeIndex::sentinel(),
            introductions: vec![],
            children: vec![],
            kind: ScopeKind::Global,
        };

        let global_scope = BlockLikeScope {
            origin: module.id,
            parent: BlockScopeIndex::sentinel(),
            introductions: vec![],
            children: vec![],
            kind: ScopeKind::Global,
        };
        self.scope_arena.intern(dummy_scope);
        let global_scope_id = self.scope_arena.intern(global_scope);

        self.scope_stack.push(global_scope_id);
        self.get_scope_mut(global_scope_id)
    }
    fn get_scope(&self, idx: BlockScopeIndex) -> &BlockLikeScope {
        self.scope_arena
            .get_by_id(idx)
            .expect("invalid block scope index")
    }
    fn get_scope_mut(&mut self, idx: BlockScopeIndex) -> &mut BlockLikeScope {
        self.scope_arena
            .get_mut_by_id(idx)
            .expect("invalid block scope index")
    }

    fn enter_scope(&mut self, origin: NodeId, kind: ScopeKind) -> &mut BlockLikeScope {
        let parent = self.current_scope_idx();
        let scope = BlockLikeScope {
            origin,
            parent,
            introductions: Vec::with_capacity(8),
            children: Vec::with_capacity(4),
            kind,
        };
        let idx = self.scope_arena.intern(scope);
        self.scope_stack.push(idx);
        self.get_scope_mut(parent).add_child(idx);
        self.get_scope_mut(idx)
    }
    fn exit_scope(&mut self) {
        assert!(!self.scope_stack.is_empty(), "cannot pop module scope");
        self.scope_stack.pop();
    }
    fn current_scope(&self) -> &BlockLikeScope {
        let idx = self
            .scope_stack
            .last()
            .expect("scope stack should not be empty");
        self.get_scope(*idx)
    }
    fn current_scope_mut(&mut self) -> &mut BlockLikeScope {
        let idx = self
            .scope_stack
            .last()
            .expect("scope stack should not be empty");
        self.get_scope_mut(*idx)
    }
    fn current_scope_idx(&self) -> BlockScopeIndex {
        self.scope_stack
            .last()
            .copied()
            .expect("scope stack should not be empty")
    }
    fn resolve(&mut self, ident: &str) -> Result<IPRScopedIdentifier, NotInScopeError> {
        let mut cur_scope_idx = self.current_scope_idx();
        let mut cur_scope = self.get_scope_mut(cur_scope_idx);

        loop {
            if let Some(found) = cur_scope.introduces(ident) {
                return Ok(found.clone());
            } else {
                if cur_scope.is_module_root() {
                    return Err(NotInScopeError {
                        ident: String::from(ident),
                        scope_stack_top: cur_scope_idx,
                    });
                }
                cur_scope_idx = cur_scope.parent;
                cur_scope = self.get_scope_mut(cur_scope_idx);
            }
        }
    }
    /// visit a branch-twig as a block if it is one, or as a regular expression if it is not.
    ///
    ///
    /// That is, push a new scope only if the twig is of kind [`ExpressionKind::Block`]
    ///
    /// as of right now, the parser only accepts blocks as the branch twigs,
    /// but this might change in the future.
    /// This method exists as a future proofing thing,
    /// as the non-block path will never be taken given the current parser implementation.
    fn visit_branch_twig(&mut self, twig: &mut IPRExpression) -> Result<(), NotInScopeError> {
        if let Some(twig) = branch_twig_as_block(twig) {
            self.enter_scope(twig.id, ScopeKind::BranchTwig);
            walk_mut_block(self, twig)?;
            self.exit_scope();
        } else {
            self.visit_expr(twig)?;
        }
        Ok(())
    }

    fn all_in_current_scope(&self) -> Vec<&IPRScopedIdentifier> {
        let mut res = Vec::with_capacity(64);
        for scope in self.scope_stack.iter() {
            let scope = self.get_scope(*scope);
            res.extend(&scope.introductions);
        }
        res
    }
}

pub struct InsertImplicitMainReturn {
    label: u32,
}

impl_nodelabeler!(InsertImplicitMainReturn, "mainreturns");

impl IPRTransfomer for InsertImplicitMainReturn {
    type TransformerError = ();
    type TransformerOk = ();
    fn visit_module(
        &mut self,
        module: &mut IPRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        if let Some(main) = module.get_main() {
            let kind = &mut main.body.tail.kind;
            if let IPRExpressionKind::Unit = kind {
                *kind = IPRExpressionKind::IntegerLiteral(0)
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::visitors::IPRTransfomer;
    use crate::ast::*;
    use crate::ast::{IPRModule, NodeLabeler};

    fn prepare_module(mut ast: IPRModule) -> (IPRModule, impl NodeLabeler) {
        let mut labeler = BareNodeLabeler::new();
        labeler.visit_module(&mut ast).unwrap();
        let labeler = ast.simplify_assignments_after(labeler);
        (ast, labeler)
    }
}
