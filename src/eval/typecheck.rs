#![allow(dead_code)]
use crate::ast::FuncDeclaration;
use crate::ast::datatype::ZeaTypeIdent;
use crate::ast::expression::{Literal, ZeaExpression};
use crate::eval::{EvaluationScheme, ZeaEvalError, ZeaValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ZeaTypeObject {
    VarType(ZeaTypeIdent),
    FuncSig(FuncDeclaration),
}

impl ZeaTypeObject {
    pub fn try_unify(
        &self,
        other: &ZeaTypeObject,
        _context: &impl EvaluationScheme,
    ) -> Result<&ZeaTypeObject, ZeaEvalError> {
        match (self, other) {
            (ZeaTypeObject::VarType(_left), ZeaTypeObject::VarType(_right)) => {
                todo!("type unification")
            }
            _ => todo!("implement try_unify on func signatures"),
        };
    }

    pub fn basic_bool_type_obj() -> ZeaTypeObject {
        ZeaTypeObject::VarType(ZeaTypeIdent::basic_bool())
    }
    pub fn basic_int_type_obj() -> ZeaTypeObject {
        ZeaTypeObject::VarType(ZeaTypeIdent::basic_int())
    }
}
pub trait TypeAnnotation {
    type Output;
    fn annotate(
        self,
        context: &impl EvaluationScheme,
    ) -> Result<ZeaValue<Self::Output>, ZeaEvalError>;
    fn peek_type(&self, context: &impl EvaluationScheme) -> Result<ZeaTypeObject, ZeaEvalError>;
}

impl TypeAnnotation for Literal {
    type Output = Literal;

    fn annotate(
        self,
        context: &impl EvaluationScheme,
    ) -> Result<ZeaValue<Self::Output>, ZeaEvalError> {
        let typ = self.peek_type(context)?;

        Ok(ZeaValue { typ, value: self })
    }

    fn peek_type(&self, _context: &impl EvaluationScheme) -> Result<ZeaTypeObject, ZeaEvalError> {
        let typ = match self {
            Self::String(_) => ZeaTypeIdent::basic_str(),
            Self::Float(_) => ZeaTypeIdent::basic_float(),
            Self::Integer(_) => ZeaTypeIdent::basic_uint(),
            Self::Boolean(_) => ZeaTypeIdent::basic_bool(),
        };
        Ok(ZeaTypeObject::VarType(typ))
    }
}
impl TypeAnnotation for ZeaExpression {
    type Output = ZeaExpression;

    fn peek_type(&self, context: &impl EvaluationScheme) -> Result<ZeaTypeObject, ZeaEvalError> {
        let typ = match self {
            ZeaExpression::Literal(l) => l.peek_type(context)?,
            ZeaExpression::Add(a, b) => {
                let a_typ = a.peek_type(context)?;
                let b_typ = b.peek_type(context)?;
                match a_typ.try_unify(&b_typ, context) {
                    Ok(result) => result.clone(),
                    Err(e) => return Err(Self::wrap_cascading_type_error(e)),
                }
            }
            ZeaExpression::IfThenElse(cond, truecase, falsecase) => {
                let _boolcond = cond
                    .peek_type(context)?
                    .try_unify(&ZeaTypeObject::basic_bool_type_obj(), context)?;

                let true_typ = truecase.peek_type(context)?;
                let false_typ = falsecase.peek_type(context)?;

                match true_typ.try_unify(&false_typ, context) {
                    Ok(result) => result.clone(),
                    Err(e) => return Err(Self::wrap_cascading_type_error(e)),
                }
            }
            _ => todo!("peek type for remaining expression variants"),
            // ZeaExpression::FuncCall(f) => {}
            // ZeaExpression::Sub(_, _) => {}
            // ZeaExpression::Mul(_, _) => {}
            // ZeaExpression::Div(_, _) => {}
            // ZeaExpression::Mod(_, _) => {}
            // ZeaExpression::Neg(_) => {}
            // ZeaExpression::LogAnd(_, _) => {}
            // ZeaExpression::LogOr(_, _) => {}
            // ZeaExpression::LogXor(_, _) => {}
            // ZeaExpression::LogNot(_) => {}
            // ZeaExpression::BitAnd(_, _) => {}
            // ZeaExpression::BitOr(_, _) => {}
            // ZeaExpression::BitXor(_, _) => {}
            // ZeaExpression::BitNot(_) => {}
            // ZeaExpression::Block(_) => {}
            // ZeaExpression::PatternMatch(_, _) => {}
            // ZeaExpression::ConditionMatch(_, _) => {}
        };
        Ok(typ)
    }
    fn annotate(
        self,
        context: &impl EvaluationScheme,
    ) -> Result<ZeaValue<Self::Output>, ZeaEvalError> {
        match self.peek_type(context) {
            Ok(typ) => Ok(ZeaValue { typ, value: self }),
            Err(e) => Err(self.wrap_outer_type_error(e)),
        }
    }
}
