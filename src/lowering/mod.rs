use crate::ast::datatype::TupleSignature;
use crate::ast::{
    AssignmentPattern, Expression, Initialisation, Literal, StructDefinition, Type,
    TypedIdentifier,
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;
pub trait LoweringExpression {}
impl LoweringExpression for Expression {}
impl LoweringExpression for LoweredExpression {}
pub enum LoweredStatement<Expr: LoweringExpression> {
    Initialisation(LoweredInitialisation<Expr>),
    Reassignment(LoweredReassignment<Expr>),
    FunctionCall(LoweredFunctionCall<Expr>),
    VoidReturn,
    Return(LoweredExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredReassignment<Expr: LoweringExpression> {
    pub assignee: String,
    pub value: Expr,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoweredFunctionCall<Expr: LoweringExpression> {
    pub name: String,
    pub arguments: Vec<Expr>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum LoweredExpression {
    FuncCall(LoweredFunctionCall<LoweredExpression>),
    Literal(Literal),
    Add(Box<LoweredExpression>, Box<LoweredExpression>),
    Sub(Box<LoweredExpression>, Box<LoweredExpression>),
    Mul(Box<LoweredExpression>, Box<LoweredExpression>),
    Div(Box<LoweredExpression>, Box<LoweredExpression>),
    Mod(Box<LoweredExpression>, Box<LoweredExpression>),
    Neg(Box<LoweredExpression>),

    LogAnd(Box<LoweredExpression>, Box<LoweredExpression>),
    LogOr(Box<LoweredExpression>, Box<LoweredExpression>),
    LogNot(Box<LoweredExpression>),

    BitAnd(Box<LoweredExpression>, Box<LoweredExpression>),
    BitOr(Box<LoweredExpression>, Box<LoweredExpression>),
    BitXor(Box<LoweredExpression>, Box<LoweredExpression>),
    BitNot(Box<LoweredExpression>),

    IfThenElse(
        Box<LoweredExpression>,
        Box<LoweredExpression>,
        Box<LoweredExpression>,
    ),
}

impl From<Literal> for LoweredExpression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredInitialisation<Expr: LoweringExpression> {
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: Expr,
}
impl<Expr: LoweringExpression> LoweredInitialisation<Expr> {
    pub fn new(typ: Option<Type>, assignee: impl Into<String>, value: Expr) -> Self {
        Self {
            typ,
            assignee: assignee.into(),
            value,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredInitialisation<Expr: LoweringExpression> {
    pub temporary: LoweredInitialisation<Expr>,
    pub unpacked_assignments: Vec<LoweredInitialisation<Expr>>,
}

impl Initialisation {
    pub fn desugar_destructuring(self) -> LoweringResult<Vec<LoweredInitialisation<Expression>>> {
        Ok(match self.assignee {
            AssignmentPattern::Identifier(assignee) => {
                vec![LoweredInitialisation::new(self.typ, assignee, self.value)]
            }

            AssignmentPattern::Tuple(_tuple) => {
                unimplemented!("implement tuple pattern assignment destructuring blabla")
            }
        })
    }
}

impl<Expr: LoweringExpression> From<DesugaredInitialisation<Expr>>
    for Vec<LoweredInitialisation<Expr>>
{
    fn from(initialiser: DesugaredInitialisation<Expr>) -> Self {
        let mut v: Vec<LoweredInitialisation<Expr>> =
            Vec::with_capacity(initialiser.unpacked_assignments.len() + 1);
        v.push(initialiser.temporary);
        v.extend(initialiser.unpacked_assignments);
        v
    }
}

pub struct TupleNamer {
    current_id: usize,
    cache: HashSet<StructDefinition>,
}

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}
impl TupleNamer {
    pub fn new() -> Self {
        Self {
            current_id: 0,
            cache: HashSet::new(),
        }
    }

    pub fn name_tuple(&mut self, _tuple: TupleSignature) -> StructDefinition {
        todo!()
    }

    pub fn tuple_is_named(_tuple: &TupleSignature) -> bool {
        todo!()
    }
}
