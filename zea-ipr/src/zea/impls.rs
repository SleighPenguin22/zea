use std::hash::{Hash, Hasher};

use crate::{
    traits::StructuralEq,
    zea::{immediate_parsed_representation::*, BinOp, NodeId, UnOp},
};

impl StructuralEq for IPRModule {
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
impl StructuralEq for IPRFuncParam {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq
    }
}
impl From<IPRTypedIdentifier> for IPRFuncParam {
    fn from(value: IPRTypedIdentifier) -> Self {
        Self {
            id: NodeId::sentinel(),
            typ: value.typ,
            name: value.name,
        }
    }
}
impl StructuralEq for IPRFunction {
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
impl From<IPRFunction> for HoistedFunctionSignature {
    fn from(value: IPRFunction) -> Self {
        HoistedFunctionSignature {
            id: value.id,
            name: value.name,
            args: value.params,
            returns: value.returns,
        }
    }
}
impl StructuralEq for IPRStatement {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.kind).eq_ignore_id(&other.kind);
        is_eq
    }
}
impl StructuralEq for IPRStatementKind {
    // initial pass
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (IPRStatementKind::Initialization(sf0), IPRStatementKind::Initialization(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRStatementKind::Reassignment(sf0), IPRStatementKind::Reassignment(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRStatementKind::FunctionCall(sf0), IPRStatementKind::FunctionCall(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRStatementKind::Return(sf0), IPRStatementKind::Return(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRStatementKind::Block(sf0), IPRStatementKind::Block(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRStatementKind::IfThenElse(sf0), IPRStatementKind::IfThenElse(of0))
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
impl StructuralEq for IPRPackedInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for IPRSimpleInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.typ).eq_ignore_id(&other.typ);
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for IPRPartiallyUnpackedInitialization {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.temporary).eq_ignore_id(&other.temporary);
        is_eq &= (self.unpacked_assignments).eq_ignore_id(&other.unpacked_assignments);
        is_eq
    }
}
impl StructuralEq for IPRInitializationKind {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (IPRInitializationKind::Packed(sf0), IPRInitializationKind::Packed(of0)) => {
                sf0.eq_ignore_id(of0)
            }

            (IPRInitializationKind::Unpacked(sf0), IPRInitializationKind::Unpacked(of0)) => {
                sf0.eq_ignore_id(of0)
            }

            _ => false,
        }
    }
}
impl StructuralEq for IPRReassignment {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.assignee).eq_ignore_id(&other.assignee);
        is_eq &= (self.value).eq_ignore_id(&other.value);
        is_eq
    }
}
impl StructuralEq for IPRFunctionCall {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.subject).eq_ignore_id(&other.subject);
        is_eq &= (self.args).eq_ignore_id(&other.args);
        is_eq
    }
}
impl StructuralEq for IPRBlockExpression {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.statements).eq_ignore_id(&other.statements);
        is_eq &= (self.tail).eq_ignore_id(&other.tail);
        is_eq
    }
}
impl StructuralEq for IPRExpression {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.kind).eq_ignore_id(&other.kind);
        is_eq
    }
}
impl StructuralEq for IPRExpressionKind {
    // initial pass
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (IPRExpressionKind::Unit, IPRExpressionKind::Unit) => true,
            (IPRExpressionKind::IntegerLiteral(sf0), IPRExpressionKind::IntegerLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::BoolLiteral(sf0), IPRExpressionKind::BoolLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::FloatLiteral(sf0), IPRExpressionKind::FloatLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::StringLiteral(sf0), IPRExpressionKind::StringLiteral(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::UnScopedIdent(sf0), IPRExpressionKind::UnScopedIdent(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::ScopedIdent(sf0), IPRExpressionKind::ScopedIdent(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (IPRExpressionKind::FunctionCall(sf0), IPRExpressionKind::FunctionCall(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }
            (
                IPRExpressionKind::BinOpExpr(sf0, sf1, sf2),
                IPRExpressionKind::BinOpExpr(of0, of1, of2),
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
            (IPRExpressionKind::UnOpExpr(sf0, sf1), IPRExpressionKind::UnOpExpr(of0, of1))
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
                IPRExpressionKind::MemberAccess(sf0, sf1),
                IPRExpressionKind::MemberAccess(of0, of1),
            ) if {
                let mut sub_items_eq = true;
                sub_items_eq &= sf0.eq_ignore_id(of0);
                sub_items_eq &= sf1.eq_ignore_id(of1);
                sub_items_eq
            } =>
            {
                true
            }
            (IPRExpressionKind::IfThenElse(sf0), IPRExpressionKind::IfThenElse(of0))
                if {
                    let mut sub_items_eq = true;
                    sub_items_eq &= sf0.eq_ignore_id(of0);
                    sub_items_eq
                } =>
            {
                true
            }

            (IPRExpressionKind::Block(sf0), IPRExpressionKind::Block(of0))
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
impl PartialEq for IPRBranch {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
// This manual impl exists because deriving it
// causes the compiler to say that the bound `&Vec<AssignmentPattern>: StructuralEq`
// is not satisfied, even though it is???
impl StructuralEq for IPRAssignmentPattern {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (IPRAssignmentPattern::Identifier(_), IPRAssignmentPattern::Identifier(_)) => true,
            (IPRAssignmentPattern::Tuple(t1), IPRAssignmentPattern::Tuple(t2)) => {
                t1.iter().zip(t2).all(|(a, b)| a.eq_ignore_id(b))
            }
            _ => false,
        }
    }
}
impl StructuralEq for IPRMatchPattern {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        match (self, other) {
            (IPRMatchPattern::Identifier(_), IPRMatchPattern::Identifier(_)) => true,
            (IPRMatchPattern::Tuple(t1), IPRMatchPattern::Tuple(t2)) => t1
                .iter()
                .zip(t2)
                .all(|(a, b)| StructuralEq::eq_ignore_id(a, b)),
            (IPRMatchPattern::UnionVariant(_, _, s3), IPRMatchPattern::UnionVariant(_, _, o3)) => {
                StructuralEq::eq_ignore_id(s3.as_ref(), o3.as_ref())
            }
            _ => false,
        }
    }
}
impl PartialEq for IPRStructDataTypeDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for IPRStructDataTypeDefinition {}
impl Hash for IPRStructDataTypeDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl StructuralEq for IPRStructDataTypeDefinition {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        let mut is_eq = true;
        is_eq &= (self.name).eq_ignore_id(&other.name);
        is_eq &= (self.members).eq_ignore_id(&other.members);
        is_eq &= (self.reorder_fields).eq_ignore_id(&other.reorder_fields);
        is_eq &= (self.alignment).eq_ignore_id(&other.alignment);
        is_eq
    }
}
impl StructuralEq for IPRTypeSpecifier {
    fn eq_ignore_id(&self, other: &Self) -> bool {
        self == other
    }
}
impl From<&str> for IPRTypeSpecifier {
    fn from(val: &str) -> Self {
        IPRTypeSpecifier::NonScalar(val.into())
    }
}

impl From<String> for IPRTypeSpecifier {
    fn from(val: String) -> Self {
        IPRTypeSpecifier::NonScalar(val)
    }
}
