pub mod altering;

use crate::zea::visitors::altering::{AssignmentSimplifier, IdentifierScoper, NodeLabeler};
use crate::zea::{immediate_parsed_representation::*, IPRScopedIdentifier};
use std::ops::Deref;

pub mod annotating;

pub trait IPRVisitor: Sized {
    type VisitorError;
    type VisitorOk: Default;
    fn visit_expr(&mut self, expr: &IPRExpression) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_expr(self, expr)
    }
    fn visit_stmt(&mut self, stmt: &IPRStatement) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_stmt(self, stmt)
    }
    fn visit_branch(&mut self, branch: &IPRBranch) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &IPRFunctionCall,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &IPRBlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &IPRTypeSpecifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &IPRInitializationBlock,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &IPRSimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_unpacked_init(self, init)
    }

    fn visit_reassignment(
        &mut self,
        reinit: &IPRReassignment,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_reassignment(self, reinit)
    }

    fn visit_init_packed(
        &mut self,
        init: &IPRPackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_packed_init(self, init)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &IPRPartiallyUnpackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &IPRScopedIdentifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        Ok(Self::VisitorOk::default())
    }
    fn visit_module(&mut self, module: &IPRModule) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &IPRFunction,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &IPRStructDataTypeDefinition,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_structdef(self, structdef)
    }
    fn visit_assignment_pattern(
        &mut self,
        pattern: &IPRAssignmentPattern,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_assignpat(self, pattern)
    }
}
pub trait Transfomer: Sized {
    type TransformerError;
    type TransformerOk: Default;
    fn visit_expr(
        &mut self,
        expr: &mut IPRExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_expr(self, expr)
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut IPRStatement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_stmt(self, stmt)
    }
    fn visit_branch(
        &mut self,
        branch: &mut IPRBranch,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &mut IPRFunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &mut IPRBlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &mut IPRTypeSpecifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &mut IPRInitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &mut IPRSimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_unpacked_init(self, init)
    }
    fn visit_init_packed(
        &mut self,
        init: &mut IPRPackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_packed_init(self, init)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &mut IPRReassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_reassignment(self, reinit)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &mut IPRPartiallyUnpackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &mut IPRScopedIdentifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        Ok(Self::TransformerOk::default())
    }
    fn visit_module(
        &mut self,
        module: &mut IPRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &mut IPRFunction,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &mut IPRStructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_structdef(self, structdef)
    }

    fn visit_assignment_pattern(
        &mut self,
        pattern: &mut IPRAssignmentPattern,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_assignpat(self, pattern)
    }
}

fn walk_expr<V: IPRVisitor>(v: &mut V, e: &IPRExpression) -> Result<V::VisitorOk, V::VisitorError> {
    match &e.kind {
        IPRExpressionKind::Unit => {}
        IPRExpressionKind::IntegerLiteral(_) => {}
        IPRExpressionKind::BoolLiteral(_) => {}
        IPRExpressionKind::FloatLiteral(_) => {}
        IPRExpressionKind::StringLiteral(_) => {}
        IPRExpressionKind::UnScopedIdent(_) => {}
        IPRExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        IPRExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        IPRExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        IPRExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        IPRExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        IPRExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }

        IPRExpressionKind::Block(eb) => {
            v.visit_block(eb)?;
        }
    }
    Ok(V::VisitorOk::default())
}
fn walk_stmt<V: IPRVisitor>(v: &mut V, s: &IPRStatement) -> Result<V::VisitorOk, V::VisitorError> {
    match &s.kind {
        IPRStatementKind::Initialization(i) => v.visit_initblock(i),
        IPRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        IPRStatementKind::FunctionCall(c) => v.visit_call(c),
        IPRStatementKind::Return(e) => v.visit_expr(e),
        IPRStatementKind::BlockTail(t) => v.visit_expr(t),

        IPRStatementKind::Block(eb) => v.visit_block(eb),
        IPRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_reassignment<V: IPRVisitor>(
    v: &mut V,
    r: &IPRReassignment,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&r.value)
}
fn walk_branch<V: IPRVisitor>(v: &mut V, b: &IPRBranch) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(b.condition.as_ref())?;
    v.visit_expr(b.true_case.as_ref())?;
    if let Some(false_case) = &b.false_case {
        v.visit_expr(false_case)?;
    }
    Ok(V::VisitorOk::default())
}
fn walk_call<V: IPRVisitor>(
    v: &mut V,
    c: &IPRFunctionCall,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(c.subject.as_ref())?;
    for a in c.args.iter() {
        v.visit_expr(a)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_block<V: IPRVisitor>(
    v: &mut V,
    block: &IPRBlockExpression,
) -> Result<V::VisitorOk, V::VisitorError> {
    for stmt in block.statements.iter() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&block.last)
}

fn walk_assignpat<V: IPRVisitor>(
    v: &mut V,
    pat: &IPRAssignmentPattern,
) -> Result<V::VisitorOk, V::VisitorError> {
    match pat {
        IPRAssignmentPattern::Identifier(_) => {}
        IPRAssignmentPattern::Tuple(t) => {
            for pat in t.iter() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::VisitorOk::default())
}

fn walk_type<V: IPRVisitor>(
    v: &mut V,
    typ: &IPRTypeSpecifier,
) -> Result<V::VisitorOk, V::VisitorError> {
    match typ {
        IPRTypeSpecifier::NonScalar(_) => {}
        IPRTypeSpecifier::Unit => {}
        IPRTypeSpecifier::Bool => {}
        IPRTypeSpecifier::Integer { .. } => {}
        IPRTypeSpecifier::Float { .. } => {}
        IPRTypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_ref())?;
        }
        IPRTypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_ref())?;
        }
        IPRTypeSpecifier::Never => {}
    };
    Ok(V::VisitorOk::default())
}

fn walk_packed_init<V: IPRVisitor>(
    v: &mut V,
    init: &IPRPackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_assignment_pattern(&init.assignee)?;
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_punpacked_init<V: IPRVisitor>(
    v: &mut V,
    init: &IPRPartiallyUnpackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_init(&init.temporary)?;
    for init in init.unpacked_assignments.iter() {
        v.visit_initblock(init)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_unpacked_init<V: IPRVisitor>(
    v: &mut V,
    init: &IPRSimpleInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_initblock<V: IPRVisitor>(
    v: &mut V,
    init: &IPRInitializationBlock,
) -> Result<V::VisitorOk, V::VisitorError> {
    match &init.kind {
        IPRInitializationKind::Packed(p) => v.visit_init_packed(p),
        IPRInitializationKind::Unpacked(u) => {
            for init in u.iter() {
                v.visit_init(init)?;
            }
            Ok(V::VisitorOk::default())
        }
    }
}

fn walk_funcdef<V: IPRVisitor>(
    v: &mut V,
    f: &IPRFunction,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_type(&f.returns)?;
    v.visit_block(&f.body)?;
    for param in f.params.iter() {
        v.visit_type(&param.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_structdef<V: IPRVisitor>(
    v: &mut V,
    s: &IPRStructDataTypeDefinition,
) -> Result<V::VisitorOk, V::VisitorError> {
    for member in s.members.iter() {
        v.visit_type(&member.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_module<V: IPRVisitor>(
    v: &mut V,
    module: &IPRModule,
) -> Result<V::VisitorOk, V::VisitorError> {
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
    e: &mut IPRExpression,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut e.kind {
        IPRExpressionKind::Unit => {}
        IPRExpressionKind::IntegerLiteral(_) => {}
        IPRExpressionKind::BoolLiteral(_) => {}
        IPRExpressionKind::FloatLiteral(_) => {}
        IPRExpressionKind::StringLiteral(_) => {}
        IPRExpressionKind::UnScopedIdent(_) => {}
        IPRExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        IPRExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        IPRExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        IPRExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        IPRExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        IPRExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }

        IPRExpressionKind::Block(eb) => {
            v.visit_block(eb)?;
        }
    };
    Ok(V::TransformerOk::default())
}
fn walk_mut_stmt<V: Transfomer>(
    v: &mut V,
    s: &mut IPRStatement,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut s.kind {
        IPRStatementKind::Initialization(i) => v.visit_initblock(i),
        IPRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        IPRStatementKind::FunctionCall(c) => v.visit_call(c),
        IPRStatementKind::Return(e) => v.visit_expr(e),
        IPRStatementKind::BlockTail(t) => v.visit_expr(t),
        IPRStatementKind::Block(eb) => v.visit_block(eb),
        IPRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_mut_reassignment<V: Transfomer>(
    v: &mut V,
    r: &mut IPRReassignment,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut r.value)
}

fn walk_mut_branch<V: Transfomer>(
    v: &mut V,
    b: &mut IPRBranch,
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
    c: &mut IPRFunctionCall,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(c.subject.as_mut())?;
    for a in c.args.iter_mut() {
        v.visit_expr(a)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_block<V: Transfomer>(
    v: &mut V,
    block: &mut IPRBlockExpression,
) -> Result<V::TransformerOk, V::TransformerError> {
    for stmt in block.statements.iter_mut() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&mut block.last)
}

fn walk_mut_assignpat<V: Transfomer>(
    v: &mut V,
    pat: &mut IPRAssignmentPattern,
) -> Result<V::TransformerOk, V::TransformerError> {
    match pat {
        IPRAssignmentPattern::Identifier(_) => {}
        IPRAssignmentPattern::Tuple(t) => {
            for pat in t.iter_mut() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_type<V: Transfomer>(
    v: &mut V,
    typ: &mut IPRTypeSpecifier,
) -> Result<V::TransformerOk, V::TransformerError> {
    match typ {
        IPRTypeSpecifier::NonScalar(_) => {}
        IPRTypeSpecifier::Unit => {}
        IPRTypeSpecifier::Bool => {}
        IPRTypeSpecifier::Integer { .. } => {}
        IPRTypeSpecifier::Float { .. } => {}
        IPRTypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_mut())?;
        }
        IPRTypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_mut())?;
        }
        IPRTypeSpecifier::Never => {}
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_packed_init<V: Transfomer>(
    v: &mut V,
    init: &mut IPRPackedInitialization,
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
    init: &mut IPRPartiallyUnpackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_init(&mut init.temporary)?;
    for init in init.unpacked_assignments.iter_mut() {
        v.visit_initblock(init)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_unpacked_init<V: Transfomer>(
    v: &mut V,
    init: &mut IPRSimpleInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut init.value)?;
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_initblock<V: Transfomer>(
    v: &mut V,
    init: &mut IPRInitializationBlock,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut init.kind {
        IPRInitializationKind::Packed(p) => v.visit_init_packed(p),
        IPRInitializationKind::Unpacked(u) => {
            for init in u.iter_mut() {
                v.visit_init(init)?;
            }
            Ok(V::TransformerOk::default())
        }
    }
}

fn walk_mut_funcdef<V: Transfomer>(
    v: &mut V,
    f: &mut IPRFunction,
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
    s: &mut IPRStructDataTypeDefinition,
) -> Result<V::TransformerOk, V::TransformerError> {
    for member in s.members.iter_mut() {
        v.visit_type(&mut member.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_module<V: Transfomer>(
    v: &mut V,
    module: &mut IPRModule,
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
impl IPRModule {
    pub fn visit_self_with<V: IPRVisitor + NodeLabeler>(
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
