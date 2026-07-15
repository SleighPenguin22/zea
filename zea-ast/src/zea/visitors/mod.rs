pub mod altering;

use crate::zea::visitors::altering::{AssignmentSimplifier, IdentifierScoper, NodeLabeler};
use crate::zea::{hir_nodes::*, HIRScopedIdentifier};
use std::ops::Deref;

pub mod annotating;

pub trait Visitor: Sized {
    type VisitorError;
    type VisitorOk: Default;
    fn visit_expr(&mut self, expr: &HIRExpression) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_expr(self, expr)
    }
    fn visit_stmt(&mut self, stmt: &HIRStatement) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_stmt(self, stmt)
    }
    fn visit_branch(&mut self, branch: &HIRBranch) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &HIRFunctionCall,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &HIRBlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &HIRTypeSpecifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &HIRInitializationBlock,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &HIRSimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_unpacked_init(self, init)
    }

    fn visit_reassignment(
        &mut self,
        reinit: &HIRReassignment,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_reassignment(self, reinit)
    }

    fn visit_init_packed(
        &mut self,
        init: &HIRPackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_packed_init(self, init)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &HIRPartiallyUnpackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &HIRScopedIdentifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        Ok(Self::VisitorOk::default())
    }
    fn visit_module(&mut self, module: &HIRModule) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &HIRFunction,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &HIRStructDataTypeDefinition,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_structdef(self, structdef)
    }
    fn visit_assignment_pattern(
        &mut self,
        pattern: &HIRAssignmentPattern,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_assignpat(self, pattern)
    }
}
pub trait Transfomer: Sized {
    type TransformerError;
    type TransformerOk: Default;
    fn visit_expr(
        &mut self,
        expr: &mut HIRExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_expr(self, expr)
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut HIRStatement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_stmt(self, stmt)
    }
    fn visit_branch(
        &mut self,
        branch: &mut HIRBranch,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &mut HIRFunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &mut HIRBlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &mut HIRTypeSpecifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &mut HIRInitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &mut HIRSimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_unpacked_init(self, init)
    }
    fn visit_init_packed(
        &mut self,
        init: &mut HIRPackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_packed_init(self, init)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &mut HIRReassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_reassignment(self, reinit)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &mut HIRPartiallyUnpackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &mut HIRScopedIdentifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        Ok(Self::TransformerOk::default())
    }
    fn visit_module(
        &mut self,
        module: &mut HIRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &mut HIRFunction,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &mut HIRStructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_structdef(self, structdef)
    }

    fn visit_assignment_pattern(
        &mut self,
        pattern: &mut HIRAssignmentPattern,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_assignpat(self, pattern)
    }
}

fn walk_expr<V: Visitor>(v: &mut V, e: &HIRExpression) -> Result<V::VisitorOk, V::VisitorError> {
    match &e.kind {
        HIRExpressionKind::Unit => {}
        HIRExpressionKind::IntegerLiteral(_) => {}
        HIRExpressionKind::BoolLiteral(_) => {}
        HIRExpressionKind::FloatLiteral(_) => {}
        HIRExpressionKind::StringLiteral(_) => {}
        HIRExpressionKind::UnScopedIdent(_) => {}
        HIRExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        HIRExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        HIRExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        HIRExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        HIRExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        HIRExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }

        HIRExpressionKind::Block(eb) => {
            v.visit_block(eb)?;
        }
    }
    Ok(V::VisitorOk::default())
}
fn walk_stmt<V: Visitor>(v: &mut V, s: &HIRStatement) -> Result<V::VisitorOk, V::VisitorError> {
    match &s.kind {
        HIRStatementKind::Initialization(i) => v.visit_initblock(i),
        HIRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        HIRStatementKind::FunctionCall(c) => v.visit_call(c),
        HIRStatementKind::Return(e) => v.visit_expr(e),
        HIRStatementKind::BlockTail(t) => v.visit_expr(t),

        HIRStatementKind::Block(eb) => v.visit_block(eb),
        HIRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_reassignment<V: Visitor>(
    v: &mut V,
    r: &HIRReassignment,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&r.value)
}
fn walk_branch<V: Visitor>(v: &mut V, b: &HIRBranch) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(b.condition.as_ref())?;
    v.visit_expr(b.true_case.as_ref())?;
    if let Some(false_case) = &b.false_case {
        v.visit_expr(false_case)?;
    }
    Ok(V::VisitorOk::default())
}
fn walk_call<V: Visitor>(v: &mut V, c: &HIRFunctionCall) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(c.subject.as_ref())?;
    for a in c.args.iter() {
        v.visit_expr(a)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_block<V: Visitor>(
    v: &mut V,
    block: &HIRBlockExpression,
) -> Result<V::VisitorOk, V::VisitorError> {
    for stmt in block.statements.iter() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&block.last)
}

fn walk_assignpat<V: Visitor>(
    v: &mut V,
    pat: &HIRAssignmentPattern,
) -> Result<V::VisitorOk, V::VisitorError> {
    match pat {
        HIRAssignmentPattern::Identifier(_) => {}
        HIRAssignmentPattern::Tuple(t) => {
            for pat in t.iter() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::VisitorOk::default())
}

fn walk_type<V: Visitor>(
    v: &mut V,
    typ: &HIRTypeSpecifier,
) -> Result<V::VisitorOk, V::VisitorError> {
    match typ {
        HIRTypeSpecifier::NonScalar(_) => {}
        HIRTypeSpecifier::Unit => {}
        HIRTypeSpecifier::Bool => {}
        HIRTypeSpecifier::Integer { .. } => {}
        HIRTypeSpecifier::Float { .. } => {}
        HIRTypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_ref())?;
        }
        HIRTypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_ref())?;
        }
        HIRTypeSpecifier::Never => {}
    };
    Ok(V::VisitorOk::default())
}

fn walk_packed_init<V: Visitor>(
    v: &mut V,
    init: &HIRPackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_assignment_pattern(&init.assignee)?;
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_punpacked_init<V: Visitor>(
    v: &mut V,
    init: &HIRPartiallyUnpackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_init(&init.temporary)?;
    for init in init.unpacked_assignments.iter() {
        v.visit_initblock(init)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_unpacked_init<V: Visitor>(
    v: &mut V,
    init: &HIRSimpleInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_initblock<V: Visitor>(
    v: &mut V,
    init: &HIRInitializationBlock,
) -> Result<V::VisitorOk, V::VisitorError> {
    match &init.kind {
        HIRInitializationKind::Packed(p) => v.visit_init_packed(p),
        HIRInitializationKind::Unpacked(u) => {
            for init in u.iter() {
                v.visit_init(init)?;
            }
            Ok(V::VisitorOk::default())
        }
    }
}

fn walk_funcdef<V: Visitor>(v: &mut V, f: &HIRFunction) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_type(&f.returns)?;
    v.visit_block(&f.body)?;
    for param in f.params.iter() {
        v.visit_type(&param.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_structdef<V: Visitor>(
    v: &mut V,
    s: &HIRStructDataTypeDefinition,
) -> Result<V::VisitorOk, V::VisitorError> {
    for member in s.members.iter() {
        v.visit_type(&member.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_module<V: Visitor>(v: &mut V, module: &HIRModule) -> Result<V::VisitorOk, V::VisitorError> {
    for glob in module.global_vars.iter() {
        v.visit_initblock(glob)?;
    }
    for func in module.functions.iter() {
        v.visit_funcdef(func)?;
    }
    for datatype in module.struct_definitions.iter() {
        v.visit_structdef(datatype)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_mut_expr<V: Transfomer>(
    v: &mut V,
    e: &mut HIRExpression,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut e.kind {
        HIRExpressionKind::Unit => {}
        HIRExpressionKind::IntegerLiteral(_) => {}
        HIRExpressionKind::BoolLiteral(_) => {}
        HIRExpressionKind::FloatLiteral(_) => {}
        HIRExpressionKind::StringLiteral(_) => {}
        HIRExpressionKind::UnScopedIdent(_) => {}
        HIRExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        HIRExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        HIRExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        HIRExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        HIRExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        HIRExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }

        HIRExpressionKind::Block(eb) => {
            v.visit_block(eb)?;
        }
    };
    Ok(V::TransformerOk::default())
}
fn walk_mut_stmt<V: Transfomer>(
    v: &mut V,
    s: &mut HIRStatement,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut s.kind {
        HIRStatementKind::Initialization(i) => v.visit_initblock(i),
        HIRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        HIRStatementKind::FunctionCall(c) => v.visit_call(c),
        HIRStatementKind::Return(e) => v.visit_expr(e),
        HIRStatementKind::BlockTail(t) => v.visit_expr(t),
        HIRStatementKind::Block(eb) => v.visit_block(eb),
        HIRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_mut_reassignment<V: Transfomer>(
    v: &mut V,
    r: &mut HIRReassignment,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut r.value)
}

fn walk_mut_branch<V: Transfomer>(
    v: &mut V,
    b: &mut HIRBranch,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(b.condition.as_mut())?;
    v.visit_expr(b.true_case.as_mut())?;
    if let Some(false_case) = &mut b.false_case {
        v.visit_expr(false_case.as_mut())?;
    }
    Ok(V::TransformerOk::default())
}
fn walk_mut_call<V: Transfomer>(
    v: &mut V,
    c: &mut HIRFunctionCall,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(c.subject.as_mut())?;
    for a in c.args.iter_mut() {
        v.visit_expr(a)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_block<V: Transfomer>(
    v: &mut V,
    block: &mut HIRBlockExpression,
) -> Result<V::TransformerOk, V::TransformerError> {
    for stmt in block.statements.iter_mut() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&mut block.last)
}

fn walk_mut_assignpat<V: Transfomer>(
    v: &mut V,
    pat: &mut HIRAssignmentPattern,
) -> Result<V::TransformerOk, V::TransformerError> {
    match pat {
        HIRAssignmentPattern::Identifier(_) => {}
        HIRAssignmentPattern::Tuple(t) => {
            for pat in t.iter_mut() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_type<V: Transfomer>(
    v: &mut V,
    typ: &mut HIRTypeSpecifier,
) -> Result<V::TransformerOk, V::TransformerError> {
    match typ {
        HIRTypeSpecifier::NonScalar(_) => {}
        HIRTypeSpecifier::Unit => {}
        HIRTypeSpecifier::Bool => {}
        HIRTypeSpecifier::Integer { .. } => {}
        HIRTypeSpecifier::Float { .. } => {}
        HIRTypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_mut())?;
        }
        HIRTypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_mut())?;
        }
        HIRTypeSpecifier::Never => {}
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_packed_init<V: Transfomer>(
    v: &mut V,
    init: &mut HIRPackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_assignment_pattern(&mut init.assignee)?;
    v.visit_expr(&mut init.value)?;
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_punpacked_init<V: Transfomer>(
    v: &mut V,
    init: &mut HIRPartiallyUnpackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_init(&mut init.temporary)?;
    for init in init.unpacked_assignments.iter_mut() {
        v.visit_initblock(init)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_unpacked_init<V: Transfomer>(
    v: &mut V,
    init: &mut HIRSimpleInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut init.value)?;
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_initblock<V: Transfomer>(
    v: &mut V,
    init: &mut HIRInitializationBlock,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut init.kind {
        HIRInitializationKind::Packed(p) => v.visit_init_packed(p),
        HIRInitializationKind::Unpacked(u) => {
            for init in u.iter_mut() {
                v.visit_init(init)?;
            }
            Ok(V::TransformerOk::default())
        }
    }
}

fn walk_mut_funcdef<V: Transfomer>(
    v: &mut V,
    f: &mut HIRFunction,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_type(&mut f.returns)?;
    v.visit_block(&mut f.body)?;
    for param in f.params.iter_mut() {
        v.visit_type(&mut param.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_structdef<V: Transfomer>(
    v: &mut V,
    s: &mut HIRStructDataTypeDefinition,
) -> Result<V::TransformerOk, V::TransformerError> {
    for member in s.members.iter_mut() {
        v.visit_type(&mut member.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_module<V: Transfomer>(
    v: &mut V,
    module: &mut HIRModule,
) -> Result<V::TransformerOk, V::TransformerError> {
    for glob in module.global_vars.iter_mut() {
        v.visit_initblock(glob)?;
    }
    for func in module.functions.iter_mut() {
        v.visit_funcdef(func)?;
    }
    for datatype in module.struct_definitions.iter_mut() {
        v.visit_structdef(datatype)?;
    }
    Ok(V::TransformerOk::default())
}
impl HIRModule {
    pub fn visit_self_with<V: Visitor + NodeLabeler>(
        &mut self,
        labeler: impl NodeLabeler,
    ) -> Result<V, V::VisitorError> {
        let mut v: V = labeler.labeler_into();
        v.visit_module(self).map(|_| v)
    }

    pub fn transform_self_with<V: Transfomer + NodeLabeler>(
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
