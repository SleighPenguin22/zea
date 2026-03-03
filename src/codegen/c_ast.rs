#![allow(unused)]
use std::hash::{Hash, Hasher};
#[derive(Debug, Clone, PartialEq)]
pub enum CExpression {
    FuncCall(CFunctionCall),
    Literal(CLiteral),
    Add(Box<CExpression>, Box<CExpression>),
    Sub(Box<CExpression>, Box<CExpression>),
    Mul(Box<CExpression>, Box<CExpression>),
    Div(Box<CExpression>, Box<CExpression>),
    Mod(Box<CExpression>, Box<CExpression>),
    Neg(Box<CExpression>),

    LogAnd(Box<CExpression>, Box<CExpression>),
    LogOr(Box<CExpression>, Box<CExpression>),
    LogNot(Box<CExpression>),

    BitAnd(Box<CExpression>, Box<CExpression>),
    BitOr(Box<CExpression>, Box<CExpression>),
    BitXor(Box<CExpression>, Box<CExpression>),
    BitNot(Box<CExpression>),

    IfThenElse(Box<IfThenElse>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    pub condition: CExpression,
    pub true_branch: CExpression,
    pub false_branch: CExpression,
}

#[derive(Debug, Clone)]
pub enum CLiteral {
    Integer(u64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl PartialEq for CLiteral {
    fn eq(&self, other: &Self) -> bool {
        if let (Self::Float(a), Self::Float(b)) = (self, other) {
            if a.is_nan() && b.is_nan() {
                return true;
            }
        }
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => unreachable!(),
        }
    }
}

impl Eq for CLiteral {}

impl Hash for CLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Float(f) if f.is_nan() => state.write(&*f64::NAN.to_ne_bytes().as_ref()),
            Self::Float(f) => state.write(&*f.to_ne_bytes().as_ref()),
            Self::Boolean(b) => b.hash(state),
            Self::String(s) => s.hash(state),
            Self::Integer(i) => i.hash(state),
        }
    }
}

pub enum CDeclaration {
    Struct(CStructDeclaration),
    Union(CUnionDeclaration),
    Enum(CEnumDeclaration),
}

pub struct CStructDeclaration(String);
pub struct CStructDefinition {
    pub name: String,
    pub members: Vec<CTypedIdentifier>,
}
pub struct CEnumDeclaration(String);
pub struct CUnionDeclaration(String);

pub struct CFunctionDeclaration(String, Vec<CTypedIdentifier>);

pub struct CTypedIdentifier(CType, String);

pub enum CType {
    Basic(String),
    Pointer(Box<CType>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CFunctionCall {
    pub name: String,
    pub args: Vec<CExpression>,
}
