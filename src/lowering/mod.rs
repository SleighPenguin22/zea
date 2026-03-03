use crate::ast::{Literal, Statement, Type};
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
    LoweredBlock(DesugaredBlockExpr),
    UnitReturn,
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
    Unit,
    FuncCall(Box<LoweringFunctionCall>),
    Ident(String),
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
    Block(Box<DesugaredBlockExpr>),
    CondMatch(Box<DesugaredCondMatch>),
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
    label: String,
    statements: Vec<LoweredStatement>,
    last: LoweredExpression,
}
impl DesugaredBlockExpr {
    pub fn insert_return_unit(
        statements: Vec<LoweredStatement>,
    ) -> (Vec<LoweredStatement>, LoweredExpression) {
        match statements.last() {
            Some(LoweredStatement::Return(expr)) => {
                (statements[..(statements.len() - 1)].to_vec(), expr.clone())
            }
            _ => (statements, LoweredExpression::Unit),
        }
    }

    pub fn new(label: String, statement_block: Vec<LoweredStatement>) -> Self {
        let (statements, last) = Self::insert_return_unit(statement_block);
        Self {
            label,
            statements,
            last,
        }
    }
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

impl DesugaredInitialisation {
    pub(crate) fn new(
        temporary: SimpleInitialisation,
        unpacked_assignments: Vec<DesugaredInitialisation>,
    ) -> DesugaredInitialisation {
        Self {
            temporary,
            unpacked_assignments,
        }
    }

    pub fn simple(typ: Option<Type>, assignee: String, value: LoweredExpression) -> Self {
        Self {
            temporary: SimpleInitialisation {
                typ,
                assignee,
                value,
            },
            unpacked_assignments: vec![],
        }
    }
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

impl Statement {}
