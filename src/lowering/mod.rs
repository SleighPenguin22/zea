///
/// This file contains all the nodes relating to the lowering of syntactic sugar
/// That is required before being able to translate to valid C code.
///
/// Any node with the `Desugared` prefix denotes some higher-level construct that is lowered
/// into the general structure of its C representation
/// Such a node is not yet able to be translated, because its actual C representation
/// might depends on the context in which it exists.
///
/// Any enum with the `Lowering` prefix represents some node
/// which had one or more of its variants converted to their `Desugared` form
///
use crate::ast::datatype::TupleSignature;
use crate::ast::{Literal, Statement, StructDefinition, Type, TypedIdentifier};
use std::collections::HashSet;
use thiserror::Error;


#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Debug, PartialEq, Clone)]
pub enum LoweredStatement {
    Initialisation(DesugaredInitialisation),
    Reassignment(LoweringReassignment),
    FunctionCall(LoweringFunctionCall),
    VoidReturn,
    Return(LoweredExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoweringReassignment {
    pub assignee: String,
    pub value: LoweredExpression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoweringFunctionCall {
    pub name: String,
    pub arguments: LoweredExpression,
}
#[derive(Debug, PartialEq, Clone)]
pub enum LoweredExpression {
    FuncCall(Box<LoweringFunctionCall>),
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
    Block(DesugaredBlockExpr),
    CondMatch(DesugaredCondMatch),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesugaredCondMatch {
    /// The label that the condmatch value gets i.e. `__cmatch0`, `__cmatch1` etc.
    /// This label must be unique to the scope of the function in which it exists
    label: usize,
    /// All of its cases, which may or may not contain a default arm.
    arms: Vec<LoweredExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesugaredBlockExpr {
    /// The label that the block expression has its value assigned to
    /// i.e. `__block0`, `__block1` etc.
    /// This label must be unique to the scope of the function in which it exists
    label: usize,
    statements: Vec<Statement>,
    last: LoweredExpression,
}

impl From<Literal> for LoweredExpression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, PartialEq, Clone)]
pub struct SimpleInitialisation {
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: LoweredExpression,
}

impl SimpleInitialisation {
    pub fn new(typ: Option<Type>, assignee: impl Into<String>, value: LoweredExpression) -> Self {
        Self {
            typ,
            assignee: assignee.into(),
            value,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredInitialisation {
    pub temporary: SimpleInitialisation,
    pub unpacked_assignments: Vec<DesugaredInitialisation>,
}

impl From<SimpleInitialisation> for DesugaredInitialisation {
    fn from(value: SimpleInitialisation) -> Self {
        Self {
            temporary: value,
            unpacked_assignments: vec![],
        }
    }
}

impl From<SimpleInitialisation> for LoweredStatement {
    fn from(value: SimpleInitialisation) -> Self {
        Self::Initialisation(value.into())
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
