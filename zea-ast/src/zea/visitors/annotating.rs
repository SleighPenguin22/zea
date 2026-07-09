use crate::helper_impls::StructuralEq;
use crate::zea::visitors::{
    walk_block, walk_expr, walk_initblock, walk_mut_funcdef, walk_unpacked_init, Visitor,
};
use crate::zea::{
    BlockExpression, Expression, ExpressionKind, FuncParam, Function, FunctionCall, IfThenElse,
    InitializationBlock, InitializationKind, Module, NodeId, PackedInitialization,
    SimpleInitialization, Statement, StatementKind,
};
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
pub struct ScopedIdentifier {
    pub ident: String,
    pub origin: NodeId,
    pub kind: ScopedIdentifierKind,
}
impl StructuralEq for ScopedIdentifier {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self.ident == other.ident && self.kind == other.kind
    }
}

impl ScopedIdentifier {
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
    pub fn from_func_param(func_param: &FuncParam) -> Self {
        ScopedIdentifier::func_param(func_param.id, func_param.name.clone())
    }
    pub fn from_funcdef(funcdef: &Function) -> Self {
        ScopedIdentifier::func_param(funcdef.id, funcdef.name.clone())
    }
    pub fn from_local_init(init: &SimpleInitialization) -> Self {
        ScopedIdentifier::local(init.id, init.assignee.clone())
    }
    pub fn from_global_init(init: &SimpleInitialization) -> Self {
        ScopedIdentifier::global(init.id, init.assignee.clone())
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
    identifiers: IndexSet<ScopedIdentifier>,
}

impl ScopeAnnotations {
    pub fn new() -> Self {
        Self {
            identifiers: IndexSet::new(),
        }
    }
    pub fn globals(&self) -> IndexSet<ScopedIdentifier> {
        self.identifiers
            .iter()
            .cloned()
            .filter(|ident| ident.kind == ScopedIdentifierKind::GlobalVar)
            .collect()
    }

    pub fn gather_idents_module(&mut self, module: &Module) {
        for glob in module.global_vars.iter() {
            self.gather_idents_global_init(glob);
        }
        for func in module.functions.iter() {
            self.gather_idents_func_def(func);
        }
    }

    fn gather_idents_local_stmt(&mut self, init: &InitializationBlock) {
        let InitializationKind::Unpacked(u) = &init.kind else {
            unreachable!()
        };
        for init in u.iter() {
            self.identifiers
                .insert(ScopedIdentifier::from_local_init(init));
        }
    }

    fn gather_idents_global_init(&mut self, init: &InitializationBlock) {
        let InitializationKind::Unpacked(u) = &init.kind else {
            unreachable!()
        };
        for init in u.iter() {
            self.identifiers
                .insert(ScopedIdentifier::from_global_init(init));
        }
    }

    fn gather_idents_func_def(&mut self, func_def: &Function) {
        self.identifiers.insert(ScopedIdentifier::func_name(
            func_def.id,
            func_def.name.clone(),
        ));
        for param in func_def.params.iter() {
            self.identifiers
                .insert(ScopedIdentifier::from_func_param(param));
        }
        for stmt in func_def.body.statements.iter() {
            self.gather_idents_stmt(stmt);
        }
    }
    fn gather_idents_stmt(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Initialization(init) => self.gather_idents_local_stmt(init),
            StatementKind::Reassignment(reinit) => self.gather_idents_expr(&reinit.value),
            StatementKind::FunctionCall(call) => self.gather_idents_call(call),
            StatementKind::Return(e) => self.gather_idents_expr(e),
            StatementKind::BlockTail(e) => self.gather_idents_expr(e),
            StatementKind::Block(eb) => self.gather_idents_block(eb),
            StatementKind::IfThenElse(ite) => self.gather_idents_branch(ite),
        }
    }

    fn gather_idents_expr(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::UnScopedIdent(_) => {}

            ExpressionKind::FunctionCall(call) => self.gather_idents_call(call),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                self.gather_idents_expr(lhs);
                self.gather_idents_expr(rhs);
            }
            ExpressionKind::UnOpExpr(_, arg) => self.gather_idents_expr(arg),
            ExpressionKind::MemberAccess(data, _) => self.gather_idents_expr(data),
            ExpressionKind::IfThenElse(ite) => self.gather_idents_branch(ite),
            ExpressionKind::Block(eb) => self.gather_idents_block(eb),
            ExpressionKind::ScopedIdent(_) => todo!("gather scoped ident"),
        }
    }
    fn gather_idents_call(&mut self, call: &FunctionCall) {
        for arg in call.args.iter() {
            self.gather_idents_expr(arg);
        }
    }
    fn gather_idents_branch(&mut self, branch: &IfThenElse) {
        self.gather_idents_expr(&branch.condition);
        self.gather_idents_expr(&branch.true_case);
        if let Some(false_case) = &branch.false_case {
            self.gather_idents_expr(false_case);
        }
    }

    fn gather_idents_block(&mut self, block: &BlockExpression) {
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
    UntypedGlobalVar(&'ast InitializationBlock),
    UnexpandedBlock(&'ast Statement),
    StrayPackedAssignment(&'ast PackedInitialization),
}
impl ASTValidator {
    pub fn blocks_expanded_in_module(module: &Module) -> Result<(), NodeId> {
        let mut validator = Self {};
        validator.visit_module(module)
    }
}

impl Visitor for ASTValidator {
    type VisitorOk = ();
    type VisitorError = NodeId;
    fn visit_block(
        &mut self,
        block: &BlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
}

pub struct ExpressionCollector {
    expressions: Vec<Expression>,
}
impl ExpressionCollector {
    pub fn over(ast: &Module) -> Self {
        let mut s = Self {
            expressions: vec![],
        };
        let _ = s.visit_module(ast);
        s
    }
    pub fn collect(self) -> Vec<Expression> {
        self.expressions
    }
}

impl Visitor for ExpressionCollector {
    type VisitorError = ();
    type VisitorOk = ();
    fn visit_expr(&mut self, expr: &Expression) -> Result<Self::VisitorOk, Self::VisitorError> {
        self.expressions.push(expr.clone());
        walk_expr(self, expr)
    }
    fn visit_init(
        &mut self,
        init: &SimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        self.expressions.push(init.value.clone());
        walk_unpacked_init(self, init)
    }

    fn visit_branch(
        &mut self,
        _branch: &IfThenElse,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        todo!("cannot yet collect expressions in branches")
    }
}
