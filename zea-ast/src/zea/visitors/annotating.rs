#![allow(clippy::new_without_default)]
use std::ops::BitAnd;

use crate::helper_impls::StructuralEq;
use crate::zea::visitors::{
    walk_block, walk_expr, walk_initblock, walk_mut_funcdef, walk_unpacked_init, Visitor,
};
use crate::zea::{hir_nodes::*, NodeId, ZeaNodeQuery};
use indexmap::{IndexMap, IndexSet};

type NodeIdMap<T> = IndexMap<NodeId, T>;
type NodeIdSet = IndexSet<NodeId>;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum ScopedIdentifierKind {
    LocalVar,
    GlobalVar,
    FunctionName,
    FunctionParam,
    ImportItem,
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct HIRScopedIdentifier {
    pub ident: String,
    pub origin: NodeId,
    pub kind: ScopedIdentifierKind,
}
impl StructuralEq for HIRScopedIdentifier {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.ident == other.ident && self.kind == other.kind
    }
}

impl HIRScopedIdentifier {
    pub fn local(origin: NodeId, ident: String) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
            kind: ScopedIdentifierKind::LocalVar,
        }
    }
    pub fn global(origin: NodeId, ident: String) -> Self {
        Self {
            origin,
            ident,
            kind: ScopedIdentifierKind::GlobalVar,
        }
    }
    pub fn func_name(origin: NodeId, ident: String) -> Self {
        Self {
            origin,
            ident,
            kind: ScopedIdentifierKind::FunctionName,
        }
    }
    pub fn func_param(origin: NodeId, ident: String) -> Self {
        Self {
            origin,
            ident,
            kind: ScopedIdentifierKind::FunctionParam,
        }
    }
    pub fn from_func_param(func_param: &HIRFuncParam) -> Self {
        HIRScopedIdentifier::func_param(func_param.id, func_param.name.clone())
    }
    pub fn from_funcdef(funcdef: &HIRFunction) -> Self {
        HIRScopedIdentifier::func_param(funcdef.id, funcdef.name.clone())
    }
    pub fn from_local_init(init: &HIRSimpleInitialization) -> Self {
        HIRScopedIdentifier::local(init.id, init.assignee.clone())
    }
    pub fn from_global_init(init: &HIRSimpleInitialization) -> Self {
        HIRScopedIdentifier::global(init.id, init.assignee.clone())
    }

    pub fn import_item(origin: NodeId, ident: String) -> Self {
        Self {
            origin,
            ident,
            kind: ScopedIdentifierKind::ImportItem,
        }
    }
}

/// The actor of the ScopeBuilder pass, this is the result of calling [`Module::annotate_scopes`]
///
/// You may query a scope-node (a node that is considered a scope)
/// for all identifiers it has in scope with [`ScopeAnnotations::get_scope`]
///
/// note that [`ScopeAnnotations::get_scope`]
/// does not verify that the given id is of a scope-node.
#[derive(Debug)]
pub struct ScopeAnnotations {
    // Map some node id to its ScopedIdentifier counterpart.
    identifiers: IndexSet<HIRScopedIdentifier>,
}

impl ScopeAnnotations {
    pub fn new() -> Self {
        Self {
            identifiers: IndexSet::new(),
        }
    }
    pub fn globals(&self) -> IndexSet<HIRScopedIdentifier> {
        self.identifiers
            .iter()
            .filter(|&ident| ident.kind == ScopedIdentifierKind::GlobalVar)
            .cloned()
            .collect()
    }

    pub fn gather_idents_module(&mut self, module: &HIRModule) {
        for glob in module.global_vars.iter() {
            self.gather_idents_global_init(glob);
        }
        for func in module.functions.iter() {
            self.gather_idents_func_def(func);
        }
    }

    fn gather_idents_local_stmt(&mut self, init: &HIRInitializationBlock) {
        let HIRInitializationKind::Unpacked(u) = &init.kind else {
            unreachable!()
        };
        for init in u.iter() {
            self.identifiers
                .insert(HIRScopedIdentifier::from_local_init(init));
        }
    }

    fn gather_idents_global_init(&mut self, init: &HIRInitializationBlock) {
        let HIRInitializationKind::Unpacked(u) = &init.kind else {
            unreachable!()
        };
        for init in u.iter() {
            self.identifiers
                .insert(HIRScopedIdentifier::from_global_init(init));
        }
    }

    fn gather_idents_func_def(&mut self, func_def: &HIRFunction) {
        self.identifiers.insert(HIRScopedIdentifier::func_name(
            func_def.id,
            func_def.name.clone(),
        ));
        for param in func_def.params.iter() {
            self.identifiers
                .insert(HIRScopedIdentifier::from_func_param(param));
        }
        for stmt in func_def.body.statements.iter() {
            self.gather_idents_stmt(stmt);
        }
    }
    fn gather_idents_stmt(&mut self, stmt: &HIRStatement) {
        match &stmt.kind {
            HIRStatementKind::Initialization(init) => self.gather_idents_local_stmt(init),
            HIRStatementKind::Reassignment(reinit) => self.gather_idents_expr(&reinit.value),
            HIRStatementKind::FunctionCall(call) => self.gather_idents_call(call),
            HIRStatementKind::Return(e) => self.gather_idents_expr(e),
            HIRStatementKind::BlockTail(e) => self.gather_idents_expr(e),
            HIRStatementKind::Block(eb) => self.gather_idents_block(eb),
            HIRStatementKind::IfThenElse(ite) => self.gather_idents_branch(ite),
        }
    }

    fn gather_idents_expr(&mut self, expr: &HIRExpression) {
        match &expr.kind {
            HIRExpressionKind::Unit => {}
            HIRExpressionKind::IntegerLiteral(_) => {}
            HIRExpressionKind::BoolLiteral(_) => {}
            HIRExpressionKind::FloatLiteral(_) => {}
            HIRExpressionKind::StringLiteral(_) => {}
            HIRExpressionKind::UnScopedIdent(_) => {}

            HIRExpressionKind::FunctionCall(call) => self.gather_idents_call(call),
            HIRExpressionKind::BinOpExpr(_, lhs, rhs) => {
                self.gather_idents_expr(lhs);
                self.gather_idents_expr(rhs);
            }
            HIRExpressionKind::UnOpExpr(_, arg) => self.gather_idents_expr(arg),
            HIRExpressionKind::MemberAccess(data, _) => self.gather_idents_expr(data),
            HIRExpressionKind::IfThenElse(ite) => self.gather_idents_branch(ite),
            HIRExpressionKind::Block(eb) => self.gather_idents_block(eb),
            HIRExpressionKind::ScopedIdent(_) => {}
        }
    }
    fn gather_idents_call(&mut self, call: &HIRFunctionCall) {
        for arg in call.args.iter() {
            self.gather_idents_expr(arg);
        }
    }
    fn gather_idents_branch(&mut self, branch: &HIRBranch) {
        self.gather_idents_expr(&branch.condition);
        self.gather_idents_expr(&branch.true_case);
        if let Some(false_case) = &branch.false_case {
            self.gather_idents_expr(false_case);
        }
    }

    fn gather_idents_block(&mut self, block: &HIRBlockExpression) {
        for stmt in block.statements.iter() {
            self.gather_idents_stmt(stmt);
        }
        self.gather_idents_expr(&block.last);
    }
}

/// This visitor will be called after each of the expansion-visitors
/// to ensure a correct AST before moving on to static analysis.
pub struct ASTValidator {}
pub enum SemanticASTViolation<'ast> {
    UntypedGlobalVar(&'ast HIRInitializationBlock),
    UnexpandedBlock(&'ast HIRStatement),
    StrayPackedAssignment(&'ast HIRPackedInitialization),
}
impl ASTValidator {
    pub fn blocks_expanded_in_module(module: &HIRModule) -> Result<(), NodeId> {
        let mut validator = Self {};
        validator.visit_module(module)
    }
}

impl Visitor for ASTValidator {
    type VisitorOk = ();
    type VisitorError = NodeId;
    fn visit_block(
        &mut self,
        block: &HIRBlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
}

pub struct ExpressionCollector {
    expressions: Vec<HIRExpression>,
}
impl ExpressionCollector {
    pub fn over(ast: &HIRModule) -> Self {
        let mut s = Self {
            expressions: vec![],
        };
        let _ = s.visit_module(ast);
        s
    }
    pub fn collect(self) -> Vec<HIRExpression> {
        self.expressions
    }
}

impl Visitor for ExpressionCollector {
    type VisitorError = ();
    type VisitorOk = ();
    fn visit_expr(&mut self, expr: &HIRExpression) -> Result<Self::VisitorOk, Self::VisitorError> {
        self.expressions.push(expr.clone());
        walk_expr(self, expr)
    }
    fn visit_init(
        &mut self,
        init: &HIRSimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        self.expressions.push(init.value.clone());
        walk_unpacked_init(self, init)
    }

    fn visit_branch(&mut self, _branch: &HIRBranch) -> Result<Self::VisitorOk, Self::VisitorError> {
        todo!("cannot yet collect expressions in branches")
    }
}

impl Visitor for ZeaNodeQuery {
    type VisitorError = ();
    type VisitorOk = Option<HIRASTNode>;
    fn visit_branch(&mut self, branch: &HIRBranch) -> Result<Self::VisitorOk, Self::VisitorError> {
        (self.id == branch.id)
            .then_some(Some(HIRASTNode::Branch(branch.clone())))
            .or_else(|| self.visit_expr(&branch.condition).ok())
            .or_else(|| self.visit_expr(&branch.true_case).ok())
            .or_else(|| {
                if let Some(twig) = &branch.false_case {
                    self.visit_expr(twig.as_ref()).ok()
                } else {
                    None
                }
            })
            .ok_or(())
    }
    fn visit_block(
        &mut self,
        block: &HIRBlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        (self.id == block.id)
            .then_some(Some(HIRASTNode::Block(block.clone())))
            .or_else(|| {
                block
                    .statements
                    .iter()
                    .find_map(|stmt| self.visit_stmt(stmt).ok())
            })
            .or_else(|| self.visit_expr(&block.last).ok())
            .ok_or(())
    }
    fn visit_module(&mut self, module: &HIRModule) -> Result<Self::VisitorOk, Self::VisitorError> {
        (self.id == module.id)
            .then_some(Some(HIRASTNode::Module(module.clone())))
            .or_else(|| {
                module
                    .global_vars
                    .iter()
                    .find_map(|glob| self.visit_initblock(glob).ok())
            })
            .or_else(|| {
                module
                    .functions
                    .iter()
                    .find_map(|f| self.visit_funcdef(f).ok())
            })
            .or_else(|| {
                module
                    .struct_definitions
                    .iter()
                    .find_map(|s| self.visit_structdef(s).ok())
            })
            .ok_or(())
    }
    fn visit_expr(&mut self, expr: &HIRExpression) -> Result<Self::VisitorOk, Self::VisitorError> {
        if self.id == expr.id {
            Ok(Some(HIRASTNode::Expression(expr.clone())))
        } else {
            match &expr.kind {
                HIRExpressionKind::Unit
                | HIRExpressionKind::IntegerLiteral(_)
                | HIRExpressionKind::BoolLiteral(_)
                | HIRExpressionKind::FloatLiteral(_)
                | HIRExpressionKind::StringLiteral(_)
                | HIRExpressionKind::UnScopedIdent(_) => Ok(None),
                HIRExpressionKind::ScopedIdent(_) => Ok(None),
                HIRExpressionKind::FunctionCall(function_call) => self.visit_call(function_call),
                HIRExpressionKind::BinOpExpr(_, expression, expression1) => self
                    .visit_expr(expression)
                    .ok()
                    .or_else(|| self.visit_expr(expression1).ok())
                    .ok_or(()),
                HIRExpressionKind::UnOpExpr(_, expression) => self.visit_expr(expression),
                HIRExpressionKind::MemberAccess(expression, _) => self.visit_expr(expression),
                HIRExpressionKind::IfThenElse(if_then_else) => self.visit_branch(if_then_else),
                HIRExpressionKind::Block(block_expression) => self.visit_block(block_expression),
            }
        }
    }
}
