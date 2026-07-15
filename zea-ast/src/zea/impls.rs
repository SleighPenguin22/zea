use std::hash::{Hash, Hasher};

use crate::{
    helper_impls::StructuralEq,
    zea::{hir_nodes::*, BinOp, NodeId, UnOp},
};

impl StructuralEq for HIRModule {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.imports).eq_ignore_id(&other.imports);
        is_eq &= (self.exports).eq_ignore_id(&other.exports);
        is_eq &= (self.global_vars).eq_ignore_id(&other.global_vars);
        is_eq &= (self.functions).eq_ignore_id(&other.functions);
        is_eq &= (self.struct_definitions).eq_ignore_id(&other.struct_definitions);
        is_eq
    }
}
impl StructuralEq for HIRFuncParam {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq
    }
}
impl From<HIRTypedIdentifier> for HIRFuncParam {
    fn from(value: HIRTypedIdentifier) -> Self {
        Self {
            id: NodeId::sentinel(),
            typ: value.typ,
            name: value.name,
        }
    }
}
impl StructuralEq for HIRFunction {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq &= (self.params).eq_ignore_id(&other.params);
        is_eq &= (self.returns).eq_ignore_id(&other.returns);
        is_eq &= (self.body).eq_ignore_id(&other.body);
        is_eq
    }
}
impl StructuralEq for HoistedFunctionSignature {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq &= (self.args).eq_ignore_id(&other.args);
        is_eq &= (self.returns).eq_ignore_id(&other.returns);
        is_eq
    }
}
impl From<HIRFunction> for HoistedFunctionSignature {
    fn from(value: HIRFunction) -> Self {
        HoistedFunctionSignature {
            id: value.id,
            name: value.name,
            args: value.params,
            returns: value.returns,
        }
    }
}
impl StructuralEq for HIRStatement {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.kind).eq_ignore_id(&other.kind);
        is_eq
    }
}
impl StructuralEq for HIRStatementKind {
    // initial pass
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (HIRStatementKind::Initialization(sf0), HIRStatementKind::Initialization(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::Reassignment(sf0), HIRStatementKind::Reassignment(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::FunctionCall(sf0), HIRStatementKind::FunctionCall(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::Return(sf0), HIRStatementKind::Return(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::BlockTail(sf0), HIRStatementKind::BlockTail(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::Block(sf0), HIRStatementKind::Block(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRStatementKind::IfThenElse(sf0), HIRStatementKind::IfThenElse(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            _ => false,
        }
    }
}
impl StructuralEq for HIRPackedInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for HIRSimpleInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for HIRPartiallyUnpackedInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.temporary).eq_ignore_id(&other.temporary);
        is_eq &= (self.unpacked_assignments).eq_ignore_id(&other.unpacked_assignments);
        is_eq
    }
}
impl StructuralEq for HIRInitializationKind {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (HIRInitializationKind::Packed(sf0), HIRInitializationKind::Packed(of0)) => {
                sf0.eq_ignore_id(of0)
            }

            (HIRInitializationKind::Unpacked(sf0), HIRInitializationKind::Unpacked(of0)) => {
                sf0.eq_ignore_id(of0)
            }

            _ => false,
        }
    }
}
impl StructuralEq for HIRReassignment {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for HIRFunctionCall {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.subject).eq_ignore_id(&other.subject);
        is_eq &= (self.args).eq_ignore_id(&other.args);
        is_eq
    }
}
impl StructuralEq for HIRBlockExpression {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.statements).eq_ignore_id(&other.statements);
        is_eq &= (self.last).eq_ignore_id(&other.last);
        is_eq
    }
}
impl StructuralEq for HIRExpression {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.kind).eq_ignore_id(&other.kind);
        is_eq
    }
}
impl StructuralEq for HIRExpressionKind {
    // initial pass
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (HIRExpressionKind::Unit, HIRExpressionKind::Unit) => true,
            (HIRExpressionKind::IntegerLiteral(sf0), HIRExpressionKind::IntegerLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::BoolLiteral(sf0), HIRExpressionKind::BoolLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::FloatLiteral(sf0), HIRExpressionKind::FloatLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::StringLiteral(sf0), HIRExpressionKind::StringLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::UnScopedIdent(sf0), HIRExpressionKind::UnScopedIdent(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::ScopedIdent(sf0), HIRExpressionKind::ScopedIdent(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (HIRExpressionKind::FunctionCall(sf0), HIRExpressionKind::FunctionCall(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (
                HIRExpressionKind::BinOpExpr(sf0, sf1, sf2),
                HIRExpressionKind::BinOpExpr(of0, of1, of2),
            ) if {
                let mut sub_items_eq = true;
                sub_items_eq &= sf0.eq_ignore_id(of0);
                sub_items_eq &= sf1.eq_ignore_id(of1);
                sub_items_eq &= sf2.eq_ignore_id(of2);
                sub_items_eq
            } =>
            {
                true
            }
            (HIRExpressionKind::UnOpExpr(sf0, sf1), HIRExpressionKind::UnOpExpr(of0, of1))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq &= sf1.eq_ignore_id(of1);
                    sub_items_eq
                } =>
            {
                true
            }
            (
                HIRExpressionKind::MemberAccess(sf0, sf1),
                HIRExpressionKind::MemberAccess(of0, of1),
            ) if {
                let mut sub_items_eq = true;
                sub_items_eq &= sf0.eq_ignore_id(of0);
                sub_items_eq &= sf1.eq_ignore_id(of1);
                sub_items_eq
            } =>
            {
                true
            }
            (HIRExpressionKind::IfThenElse(sf0), HIRExpressionKind::IfThenElse(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }

            (HIRExpressionKind::Block(sf0), HIRExpressionKind::Block(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            _ => false,
        }
    }
}
impl StructuralEq for BinOp {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}
impl StructuralEq for UnOp {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (UnOp::Neg, UnOp::Neg) | (UnOp::LogNot, UnOp::LogNot) | (UnOp::BitNot, UnOp::BitNot)
        )
    }
}
impl PartialEq for HIRBranch {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
// This manual impl exists because deriving it
// causes the compiler to say that the bound `&Vec<AssignmentPattern>: StructuralEq`
// is not satisfied, even though it is???
impl StructuralEq for HIRAssignmentPattern {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (HIRAssignmentPattern::Identifier(_), HIRAssignmentPattern::Identifier(_)) => true,
            (HIRAssignmentPattern::Tuple(t1), HIRAssignmentPattern::Tuple(t2)) => {
                t1.iter().zip(t2).all(|(a, b)| a.eq_ignore_id(b))
            }
            _ => false,
        }
    }
}
impl StructuralEq for HIRMatchPattern {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (HIRMatchPattern::Identifier(_), HIRMatchPattern::Identifier(_)) => true,
            (HIRMatchPattern::Tuple(t1), HIRMatchPattern::Tuple(t2)) => t1
                .iter()
                .zip(t2)
                .all(|(a, b)| StructuralEq::eq_ignore_id(a, b)),
            (HIRMatchPattern::UnionVariant(_, _, s3), HIRMatchPattern::UnionVariant(_, _, o3)) => {
                StructuralEq::eq_ignore_id(s3.as_ref(), o3.as_ref())
            }
            _ => false,
        }
    }
}
impl PartialEq for HIRStructDataTypeDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for HIRStructDataTypeDefinition {}
impl Hash for HIRStructDataTypeDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl StructuralEq for HIRStructDataTypeDefinition {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq &= (self.members).eq_ignore_id(&other.members);
        is_eq &= (self.reorder_fields).eq_ignore_id(&other.reorder_fields);
        is_eq &= (self.alignment).eq_ignore_id(&other.alignment);
        is_eq
    }
}
impl StructuralEq for HIRTypeSpecifier {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}
impl From<&str> for HIRTypeSpecifier {
    fn from(val: &str) -> Self {
        HIRTypeSpecifier::NonScalar(val.into())
    }
}

impl From<String> for HIRTypeSpecifier {
    fn from(val: String) -> Self {
        HIRTypeSpecifier::NonScalar(val)
    }
}
