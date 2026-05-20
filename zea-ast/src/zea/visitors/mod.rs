pub mod altering;

use crate::zea::visitors::altering::{AssignmentSimplifier, BlockExpander, NodeLabeler};
use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, InitializationBlock, InitializationKind, Module, PackedInitialization,
    PartiallyUnpackedInitialization, Reassignment, ScopedIdentifier, SimpleInitialization,
    Statement, StatementBlock, StatementKind, StructDataTypeDefinition, TypeSpecifier,
};
use std::ops::Deref;

pub mod annotating;

pub trait Visitor: Sized {
    type VisitorError;
    type VisitorOk: Default;
    fn visit_expr(&mut self, expr: &Expression) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_expr(self, expr)
    }
    fn visit_stmt(&mut self, stmt: &Statement) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_stmt(self, stmt)
    }
    fn visit_branch(&mut self, branch: &IfThenElse) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_branch(self, branch)
    }
    fn visit_call(&mut self, call: &FunctionCall) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_call(self, call)
    }
    fn visit_sugared_block(
        &mut self,
        sugared_block: &StatementBlock,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_sugared_block(self, sugared_block)
    }

    fn visit_block(
        &mut self,
        block: &ExpandedBlockExpr,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_block(self, block)
    }
    fn visit_type(&mut self, typ: &TypeSpecifier) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &InitializationBlock,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &SimpleInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_unpacked_init(self, init)
    }

    fn visit_reassignment(
        &mut self,
        reinit: &Reassignment,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_reassignment(self, reinit)
    }

    fn visit_init_packed(
        &mut self,
        init: &PackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_packed_init(self, init)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &PartiallyUnpackedInitialization,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &ScopedIdentifier,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        Ok(Self::VisitorOk::default())
    }
    fn visit_module(&mut self, module: &Module) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_module(self, module)
    }

    fn visit_funcdef(&mut self, funcdef: &Function) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &StructDataTypeDefinition,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_structdef(self, structdef)
    }
    fn visit_assignment_pattern(
        &mut self,
        pattern: &AssignmentPattern,
    ) -> Result<Self::VisitorOk, Self::VisitorError> {
        walk_assignpat(self, pattern)
    }
}
pub trait Transfomer: Sized {
    type TransformerError;
    type TransformerOk: Default;
    fn visit_expr(
        &mut self,
        expr: &mut Expression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_expr(self, expr)
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut Statement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_stmt(self, stmt)
    }
    fn visit_branch(
        &mut self,
        branch: &mut IfThenElse,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &mut FunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_call(self, call)
    }
    fn visit_sugared_block(
        &mut self,
        sugared_block: &mut StatementBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_sugared_block(self, sugared_block)
    }

    fn visit_block(
        &mut self,
        block: &mut ExpandedBlockExpr,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_block(self, block)
    }
    fn visit_type(
        &mut self,
        typ: &mut TypeSpecifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_type(self, typ)
    }
    fn visit_initblock(
        &mut self,
        init: &mut InitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_initblock(self, init)
    }
    fn visit_init(
        &mut self,
        init: &mut SimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_unpacked_init(self, init)
    }
    fn visit_init_packed(
        &mut self,
        init: &mut PackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_packed_init(self, init)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &mut Reassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_reassignment(self, reinit)
    }
    fn visit_init_punpacked(
        &mut self,
        init: &mut PartiallyUnpackedInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_punpacked_init(self, init)
    }

    fn visit_scoped_identifier(
        &mut self,
        _ident: &mut ScopedIdentifier,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        Ok(Self::TransformerOk::default())
    }
    fn visit_module(
        &mut self,
        module: &mut Module,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_module(self, module)
    }

    fn visit_funcdef(
        &mut self,
        funcdef: &mut Function,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_funcdef(self, funcdef)
    }

    fn visit_structdef(
        &mut self,
        structdef: &mut StructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_structdef(self, structdef)
    }

    fn visit_assignment_pattern(
        &mut self,
        pattern: &mut AssignmentPattern,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        walk_mut_assignpat(self, pattern)
    }
}

fn walk_expr<'v, V: Visitor>(v: &mut V, e: &Expression) -> Result<V::VisitorOk, V::VisitorError> {
    match &e.kind {
        ExpressionKind::Unit => {}
        ExpressionKind::IntegerLiteral(_) => {}
        ExpressionKind::BoolLiteral(_) => {}
        ExpressionKind::FloatLiteral(_) => {}
        ExpressionKind::StringLiteral(_) => {}
        ExpressionKind::UnScopedIdent(_) => {}
        ExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        ExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        ExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        ExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        ExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        ExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }
        ExpressionKind::Block(b) => {
            v.visit_sugared_block(b)?;
        }
        ExpressionKind::ExpandedBlock(eb) => {
            v.visit_block(eb)?;
        }
    }
    Ok(V::VisitorOk::default())
}
fn walk_stmt<'v, V: Visitor>(v: &mut V, s: &Statement) -> Result<V::VisitorOk, V::VisitorError> {
    match &s.kind {
        StatementKind::Initialization(i) => v.visit_initblock(i),
        StatementKind::Reassignment(r) => v.visit_reassignment(r),
        StatementKind::FunctionCall(c) => v.visit_call(c),
        StatementKind::Return(e) => v.visit_expr(e),
        StatementKind::BlockTail(t) => v.visit_expr(t),
        StatementKind::SugaredBlock(b) => v.visit_sugared_block(b),
        StatementKind::Block(eb) => v.visit_block(eb),
        StatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_reassignment<V: Visitor>(
    v: &mut V,
    r: &Reassignment,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&r.value)
}
fn walk_branch<V: Visitor>(v: &mut V, b: &IfThenElse) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(b.condition.as_ref())?;
    v.visit_expr(b.true_case.as_ref())?;
    if let Some(false_case) = &b.false_case {
        v.visit_expr(false_case)?;
    }
    Ok(V::VisitorOk::default())
}
fn walk_call<V: Visitor>(v: &mut V, c: &FunctionCall) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(c.subject.as_ref())?;
    for a in c.args.iter() {
        v.visit_expr(a)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_sugared_block<V: Visitor>(
    v: &mut V,
    block: &StatementBlock,
) -> Result<V::VisitorOk, V::VisitorError> {
    for stmt in block.statements.iter() {
        v.visit_stmt(stmt)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_block<V: Visitor>(
    v: &mut V,
    block: &ExpandedBlockExpr,
) -> Result<V::VisitorOk, V::VisitorError> {
    for stmt in block.statements.iter() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&block.last)
}

fn walk_assignpat<V: Visitor>(
    v: &mut V,
    pat: &AssignmentPattern,
) -> Result<V::VisitorOk, V::VisitorError> {
    match pat {
        AssignmentPattern::Identifier(_) => {}
        AssignmentPattern::Tuple(t) => {
            for pat in t.iter() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::VisitorOk::default())
}

fn walk_type<V: Visitor>(v: &mut V, typ: &TypeSpecifier) -> Result<V::VisitorOk, V::VisitorError> {
    match typ {
        TypeSpecifier::Basic(_) => {}
        TypeSpecifier::Unit => {}
        TypeSpecifier::Bool => {}
        TypeSpecifier::Integer { .. } => {}
        TypeSpecifier::Float { .. } => {}
        TypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_ref())?;
        }
        TypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_ref())?;
        }
        TypeSpecifier::Never => {}
    };
    Ok(V::VisitorOk::default())
}

fn walk_packed_init<V: Visitor>(
    v: &mut V,
    init: &PackedInitialization,
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
    init: &PartiallyUnpackedInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_init(&init.temporary)?;
    for init in init.unpacked_assignments.iter() {
        v.visit_initblock(init)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_unpacked_init<V: Visitor>(
    v: &mut V,
    init: &SimpleInitialization,
) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_expr(&init.value)?;
    if let Some(t) = &init.typ {
        v.visit_type(t)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_initblock<V: Visitor>(
    v: &mut V,
    init: &InitializationBlock,
) -> Result<V::VisitorOk, V::VisitorError> {
    match &init.kind {
        InitializationKind::Packed(p) => v.visit_init_packed(p),
        InitializationKind::Unpacked(u) => {
            for init in u.iter() {
                v.visit_init(init)?;
            }
            Ok(V::VisitorOk::default())
        }
    }
}

fn walk_funcdef<V: Visitor>(v: &mut V, f: &Function) -> Result<V::VisitorOk, V::VisitorError> {
    v.visit_type(&f.returns)?;
    v.visit_sugared_block(&f.body)?;
    for param in f.params.iter() {
        v.visit_type(&param.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_structdef<V: Visitor>(
    v: &mut V,
    s: &StructDataTypeDefinition,
) -> Result<V::VisitorOk, V::VisitorError> {
    for member in s.members.iter() {
        v.visit_type(&member.typ)?;
    }
    Ok(V::VisitorOk::default())
}

fn walk_module<V: Visitor>(v: &mut V, module: &Module) -> Result<V::VisitorOk, V::VisitorError> {
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

fn walk_mut_expr<'v, V: Transfomer>(
    v: &mut V,
    e: &mut Expression,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut e.kind {
        ExpressionKind::Unit => {}
        ExpressionKind::IntegerLiteral(_) => {}
        ExpressionKind::BoolLiteral(_) => {}
        ExpressionKind::FloatLiteral(_) => {}
        ExpressionKind::StringLiteral(_) => {}
        ExpressionKind::UnScopedIdent(_) => {}
        ExpressionKind::ScopedIdent(i) => {
            v.visit_scoped_identifier(i)?;
        }
        ExpressionKind::FunctionCall(call) => {
            v.visit_call(call)?;
        }
        ExpressionKind::BinOpExpr(_, l, r) => {
            v.visit_expr(l)?;
            v.visit_expr(r)?;
        }
        ExpressionKind::UnOpExpr(_, a) => {
            v.visit_expr(a)?;
        }
        ExpressionKind::MemberAccess(d, _) => {
            v.visit_expr(d)?;
        }
        ExpressionKind::IfThenElse(ite) => {
            v.visit_branch(ite)?;
        }
        ExpressionKind::Block(b) => {
            v.visit_sugared_block(b)?;
        }
        ExpressionKind::ExpandedBlock(eb) => {
            v.visit_block(eb)?;
        }
    };
    Ok(V::TransformerOk::default())
}
fn walk_mut_stmt<'v, V: Transfomer>(
    v: &mut V,
    s: &mut Statement,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut s.kind {
        StatementKind::Initialization(i) => v.visit_initblock(i),
        StatementKind::Reassignment(r) => v.visit_reassignment(r),
        StatementKind::FunctionCall(c) => v.visit_call(c),
        StatementKind::Return(e) => v.visit_expr(e),
        StatementKind::BlockTail(t) => v.visit_expr(t),
        StatementKind::SugaredBlock(b) => v.visit_sugared_block(b),
        StatementKind::Block(eb) => v.visit_block(eb),
        StatementKind::IfThenElse(ite) => v.visit_branch(ite),
    }
}

fn walk_mut_reassignment<V: Transfomer>(
    v: &mut V,
    r: &mut Reassignment,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut r.value)
}

fn walk_mut_branch<V: Transfomer>(
    v: &mut V,
    b: &mut IfThenElse,
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
    c: &mut FunctionCall,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(c.subject.as_mut())?;
    for a in c.args.iter_mut() {
        v.visit_expr(a)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_sugared_block<V: Transfomer>(
    v: &mut V,
    block: &mut StatementBlock,
) -> Result<V::TransformerOk, V::TransformerError> {
    for stmt in block.statements.iter_mut() {
        v.visit_stmt(stmt)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_block<V: Transfomer>(
    v: &mut V,
    block: &mut ExpandedBlockExpr,
) -> Result<V::TransformerOk, V::TransformerError> {
    for stmt in block.statements.iter_mut() {
        v.visit_stmt(stmt)?;
    }
    v.visit_expr(&mut block.last)
}

fn walk_mut_assignpat<V: Transfomer>(
    v: &mut V,
    pat: &mut AssignmentPattern,
) -> Result<V::TransformerOk, V::TransformerError> {
    match pat {
        AssignmentPattern::Identifier(_) => {}
        AssignmentPattern::Tuple(t) => {
            for pat in t.iter_mut() {
                v.visit_assignment_pattern(pat)?;
            }
        }
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_type<V: Transfomer>(
    v: &mut V,
    typ: &mut TypeSpecifier,
) -> Result<V::TransformerOk, V::TransformerError> {
    match typ {
        TypeSpecifier::Basic(_) => {}
        TypeSpecifier::Unit => {}
        TypeSpecifier::Bool => {}
        TypeSpecifier::Integer { .. } => {}
        TypeSpecifier::Float { .. } => {}
        TypeSpecifier::Pointer(t) => {
            v.visit_type(t.as_mut())?;
        }
        TypeSpecifier::ArrayOf(t) => {
            v.visit_type(t.as_mut())?;
        }
        TypeSpecifier::Never => {}
    };
    Ok(V::TransformerOk::default())
}

fn walk_mut_packed_init<V: Transfomer>(
    v: &mut V,
    init: &mut PackedInitialization,
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
    init: &mut PartiallyUnpackedInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_init(&mut init.temporary)?;
    for init in init.unpacked_assignments.iter_mut() {
        v.visit_initblock(init)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_unpacked_init<V: Transfomer>(
    v: &mut V,
    init: &mut SimpleInitialization,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_expr(&mut init.value)?;
    if let Some(t) = &mut init.typ {
        v.visit_type(t)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_initblock<V: Transfomer>(
    v: &mut V,
    init: &mut InitializationBlock,
) -> Result<V::TransformerOk, V::TransformerError> {
    match &mut init.kind {
        InitializationKind::Packed(p) => v.visit_init_packed(p),
        InitializationKind::Unpacked(u) => {
            for init in u.iter_mut() {
                v.visit_init(init)?;
            }
            Ok(V::TransformerOk::default())
        }
    }
}

fn walk_mut_funcdef<V: Transfomer>(
    v: &mut V,
    f: &mut Function,
) -> Result<V::TransformerOk, V::TransformerError> {
    v.visit_type(&mut f.returns)?;
    v.visit_sugared_block(&mut f.body)?;
    for param in f.params.iter_mut() {
        v.visit_type(&mut param.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_structdef<V: Transfomer>(
    v: &mut V,
    s: &mut StructDataTypeDefinition,
) -> Result<V::TransformerOk, V::TransformerError> {
    for member in s.members.iter_mut() {
        v.visit_type(&mut member.typ)?;
    }
    Ok(V::TransformerOk::default())
}

fn walk_mut_module<V: Transfomer>(
    v: &mut V,
    module: &mut Module,
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
impl Module {
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

    pub fn expand_blocks_with(&mut self, labeler: impl NodeLabeler) -> BlockExpander {
        self.transform_self_with(labeler).unwrap()
    }

    pub fn simplify_assignments_after(&mut self, labeler: impl NodeLabeler) -> AssignmentSimplifier {
        self.transform_self_with(labeler).unwrap()
    }
}
