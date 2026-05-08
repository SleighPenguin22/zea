use crate::helper_impls::StructuralEq;
use crate::zea::{
    ExpandedBlockExpr, Expression, ExpressionKind, FuncParam, Function, FunctionCall, IfThenElse,
    Initialization, InitializationKind, Module, PackedInitialization, SimpleInitialization,
    Statement, StatementKind,
};
use indexmap::IndexSet;
use std::collections::HashSet;

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
    pub origin: usize,
    pub kind: ScopedIdentifierKind,
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
    pub fn from_func_param(func_param: &FuncParam) -> Self {
        ScopedIdentifier::func_param(func_param.id, func_param.name.as_ref())
    }
    pub fn from_local_init(init: &SimpleInitialization) -> Self {
        ScopedIdentifier::local(init.id, init.assignee.as_ref())
    }
    pub fn from_global_init(init: &SimpleInitialization) -> Self {
        ScopedIdentifier::global(init.id, init.assignee.as_ref())
    }

    pub fn import_item(origin: usize, ident: &str) -> Self {
        Self {
            origin,
            ident: ident.to_string(),
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
pub struct ScopeAnnotations {
    // Map some node id to its ScopedIdentifier counterpart.
    identifiers: IndexSet<ScopedIdentifier>,
}

impl ScopedIdentifier {}

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

    fn gather_idents_local_stmt(&mut self, init: &Initialization) {
        let InitializationKind::Unpacked(u) = &init.kind else {
            unreachable!()
        };
        for init in u.iter() {
            self.identifiers
                .insert(ScopedIdentifier::from_local_init(init));
        }
    }

    fn gather_idents_global_init(&mut self, init: &Initialization) {
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
            func_def.name.as_ref(),
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
            StatementKind::ExpandedBlock(eb) => self.gather_idents_block(eb),
            StatementKind::IfThenElse(ite) => self.gather_idents_branch(ite),

            StatementKind::Block(_) => unreachable!(),
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
            ExpressionKind::ExpandedBlock(eb) => self.gather_idents_block(eb),

            ExpressionKind::Block(_) | ExpressionKind::ScopedIdent(_) => unreachable!(),
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

    fn gather_idents_block(&mut self, block: &ExpandedBlockExpr) {
        for stmt in block.statements.iter() {
            self.gather_idents_stmt(stmt);
        }
        self.gather_idents_expr(&block.last);
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
