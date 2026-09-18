//! This module contains the visitor and transformer skeleton for IPR passes
//!
//! implementing a visitor is as easy as implementing the [`IPRVisitor`] trait,
//! and then overriding any of the `visit_[node]` methods
//! to specify behaviour when encountering a given node.
//!
//! Any visitor or transformer that synthesizes new IPR nodes, must give that node a unique ID
//! In order to do that, a visitor must implement the [`NodeLabeler`] trait,
//! which allows it to generate such a unique ID.
//!
//! These ID's are just `u32`'s. To hold the invariant that each ID is unique,
//! the [`NodeLabeler::labeler_from`] method can be used
//! to pass the last used ID as the starting label for the new labeler.
//!
//! The [`impl_nodelabeler`] macro is provided to implement the trait
//! for some struct with a `label: usize` field

use crate::ast::ipr_walkers::transformers::{AssignmentExpander, IdentifierScoper, NodeLabeler};
use crate::ast::{IPRScopedIdentifier, ipr::*};
use std::ops::Deref;

pub mod transformers;
pub mod visitors;

pub trait IPRVisitor<'m>: Sized {
    type VisitorError;
    type VisitorOk: Default;
    fn visit_expr(
        &mut self,
        expr: &'m IPRExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_expr(self, expr)
    }
    fn visit_stmt(
        &mut self,
        stmt: &'m IPRStatement,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_stmt(self, stmt)
    }
    fn visit_branch(
        &mut self,
        branch: &'m IPRBranch,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &'m IPRFunctionCall,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &'m IPRBlockExpression,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &'m IPRTypeSpecifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &'m IPRInitializationBlock,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &'m IPRSimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_unpacked_init(self, init)
    }

    fn visit_reassignment(
        &mut self,
        reinit: &'m IPRReassignment,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_reassignment(self, reinit)
    }

    fn visit_init_packed(
        &mut self,
        init: &'m IPRPackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_packed_init(self, init)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &'m IPRPartiallyUnpackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &'m IPRScopedIdentifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        Ok(Self::VisitorOk::default())
    }
    fn visit_module(
        &mut self,
        module: &'m IPRModule,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &'m IPRFunction,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_funcdef(self, funcdef)
    }
    fn visit_funcparam(
        &mut self,
        param: &'m IPRFuncParam,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_funcparam(self, param)
    }

    fn visit_structdef(
        &mut self,
        structdef: &'m IPRStructDataTypeDefinition,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_structdef(self, structdef)
    }
    fn visit_assignment_pattern(
        &mut self,
        pattern: &'m IPRAssignmentPattern,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_assignpat(self, pattern)
    }
}

pub trait IPRTransfomer<'m>: Sized {
    type TransformerError;
    type TransformerOk: Default;
    fn visit_expr(
        &mut self,
        expr: &'m mut IPRExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_expr(self, expr)
    }
    fn visit_stmt(
        &mut self,
        stmt: &'m mut IPRStatement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_stmt(self, stmt)
    }
    fn visit_branch(
        &mut self,
        branch: &'m mut IPRBranch,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &'m mut IPRFunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_call(self, call)
    }

    fn visit_block(
        &mut self,
        block: &'m mut IPRBlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &'m mut IPRTypeSpecifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &'m mut IPRInitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &'m mut IPRSimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_unpacked_init(self, init)
    }
    fn visit_init_packed(
        &mut self,
        init: &'m mut IPRPackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_packed_init(self, init)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &'m mut IPRReassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_reassignment(self, reinit)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &'m mut IPRPartiallyUnpackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &'m mut IPRScopedIdentifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        Ok(Self::TransformerOk::default())
    }
    fn visit_module(
        &mut self,
        module: &'m mut IPRModule,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &'m mut IPRFunction,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_funcdef(self, funcdef)
    }
    fn visit_funcparam(
        &mut self,
        param: &'m mut IPRFuncParam,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_funcparam(self, param)
    }

    fn visit_structdef(
        &mut self,
        structdef: &'m mut IPRStructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_structdef(self, structdef)
    }

    fn visit_assignment_pattern(
        &mut self,
        pattern: &'m mut IPRAssignmentPattern,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_assignpat(self, pattern)
    }
}

fn walk_expr<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    e: &'m IPRExpression,
) -> Result<V::VisitorOk, V::VisitorError> {
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
fn walk_stmt<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    s: &'m IPRStatement,
) -> Result<V::VisitorOk, V::VisitorError> {
    match &s.kind {
        IPRStatementKind::Initialization(i) => v.visit_initblock(i),
        IPRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        IPRStatementKind::FunctionCall(c) => v.visit_call(c),
        IPRStatementKind::Return(e) => v.visit_expr(e),

        IPRStatementKind::Block(eb) => v.visit_block(eb),
        IPRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_reassignment<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    r: &'m IPRReassignment,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&r.value)
}
fn walk_branch<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    b: &'m IPRBranch,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(b.condition.as_ref())?;
    v.visit_expr(b.true_case.as_ref())?;
    if let Some(false_case) = &b.false_case {
        v.visit_expr(false_case)?;
    }
    Ok(V::VisitorOk::default())
}
fn walk_call<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    c: &'m IPRFunctionCall,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(c.subject.as_ref())?;
    for a in c.args.iter() {
        v.visit_expr(a)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_block<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    block: &'m IPRBlockExpression,
) -> Result<V::VisitorOk, V::VisitorError> {
    for stmt in block.statements.iter() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&block.tail)
}

fn walk_assignpat<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    pat: &'m IPRAssignmentPattern,
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

fn walk_type<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    typ: &'m IPRTypeSpecifier,
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

fn walk_packed_init<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    init: &'m IPRPackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_assignment_pattern(&init.assignee)?;
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_punpacked_init<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    init: &'m IPRPartiallyUnpackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_init(&init.temporary)?;
    for init in init.unpacked_assignments.iter() {
        v.visit_initblock(init)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_unpacked_init<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    init: &'m IPRSimpleInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_initblock<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    init: &'m IPRInitializationBlock,
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

fn walk_funcdef<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    f: &'m IPRFunction,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_type(&f.returns)?;
    v.visit_block(&f.body)?;
    for param in f.params.iter() {
        v.visit_type(&param.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_funcparam<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    f: &'m IPRFuncParam,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_type(&f.typ)
}
fn walk_structdef<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    s: &'m IPRStructDataTypeDefinition,
) -> Result<V::VisitorOk, V::VisitorError> {
    for member in s.members.iter() {
        v.visit_type(&member.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_module<'m, V: IPRVisitor<'m>>(
    v: &mut V,
    module: &'m IPRModule,
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

fn walk_mut_expr<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    e: &'m mut IPRExpression,
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
fn walk_mut_stmt<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    s: &'m mut IPRStatement,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut s.kind {
        IPRStatementKind::Initialization(i) => v.visit_initblock(i),
        IPRStatementKind::Reassignment(r) => v.visit_reassignment(r),
        IPRStatementKind::FunctionCall(c) => v.visit_call(c),
        IPRStatementKind::Return(e) => v.visit_expr(e),
        IPRStatementKind::Block(eb) => v.visit_block(eb),
        IPRStatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_mut_reassignment<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    r: &'m mut IPRReassignment,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut r.value)
}

fn walk_mut_branch<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    b: &'m mut IPRBranch,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(b.condition.as_mut())?;
    v.visit_expr(b.true_case.as_mut())?;
    if let Some(false_case) = &mut b.false_case {
        v.visit_expr(false_case.as_mut())?;
    }
    Ok(V::TransformerOk::default())
}
fn walk_mut_call<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    c: &'m mut IPRFunctionCall,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(c.subject.as_mut())?;
    for a in c.args.iter_mut() {
        v.visit_expr(a)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_block<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    block: &'m mut IPRBlockExpression,
) -> Result<V::TransformerOk, V::TransformerError> {
    for stmt in block.statements.iter_mut() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&mut block.tail)
}

fn walk_mut_assignpat<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    pat: &'m mut IPRAssignmentPattern,
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

fn walk_mut_type<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    typ: &'m mut IPRTypeSpecifier,
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

fn walk_mut_packed_init<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    init: &'m mut IPRPackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_assignment_pattern(&mut init.assignee)?;
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    v.visit_expr(&mut init.value)
}

fn walk_mut_punpacked_init<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    init: &'m mut IPRPartiallyUnpackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_init(&mut init.temporary)?;
    for init in init.unpacked_assignments.iter_mut() {
        v.visit_initblock(init)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_unpacked_init<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    init: &'m mut IPRSimpleInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    v.visit_expr(&mut init.value)
}

fn walk_mut_initblock<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    init: &'m mut IPRInitializationBlock,
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

fn walk_mut_funcdef<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    f: &'m mut IPRFunction,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_type(&mut f.returns)?;
    v.visit_block(&mut f.body)?;
    for param in f.params.iter_mut() {
        v.visit_type(&mut param.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_funcparam<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    f: &'m mut IPRFuncParam,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_type(&mut f.typ)
}
fn walk_mut_structdef<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    s: &'m mut IPRStructDataTypeDefinition,
) -> Result<V::TransformerOk, V::TransformerError> {
    for member in s.members.iter_mut() {
        v.visit_type(&mut member.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_module<'m, V: IPRTransfomer<'m>>(
    v: &mut V,
    module: &'m mut IPRModule,
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
