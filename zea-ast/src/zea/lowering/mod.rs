use crate::zea::expression::Literal;
use crate::zea::statement::Statement;
use crate::zea::Type;
use thiserror::Error;

pub mod nodeexpansion;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoweringError {
    #[error("{0}")]
    Other(String),
}
pub type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Debug, PartialEq, Clone)]
pub enum ExpandedStatement {
    Initialisation(ExpandedInitialisation),
    Reassignment(ExpandedReassignment),
    FunctionCall(LoweringFunctionCall),
    LoweredBlock(ExpandedBlockExpr),
    UnitReturn,
    Return(ExpandedExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpandedReassignment {
    pub assignee: String,
    pub value: ExpandedExpression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoweringFunctionCall {
    pub name: String,
    pub arguments: Vec<ExpandedExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ExpandedExpression {
    Unit,
    FuncCall(Box<LoweringFunctionCall>),
    Ident(String),
    Literal(Literal),
    Add(Box<ExpandedExpression>, Box<ExpandedExpression>),
    Sub(Box<ExpandedExpression>, Box<ExpandedExpression>),
    Mul(Box<ExpandedExpression>, Box<ExpandedExpression>),
    Div(Box<ExpandedExpression>, Box<ExpandedExpression>),
    Mod(Box<ExpandedExpression>, Box<ExpandedExpression>),
    Neg(Box<ExpandedExpression>),

    LogAnd(Box<ExpandedExpression>, Box<ExpandedExpression>),
    LogOr(Box<ExpandedExpression>, Box<ExpandedExpression>),
    LogNot(Box<ExpandedExpression>),

    BitAnd(Box<ExpandedExpression>, Box<ExpandedExpression>),
    BitOr(Box<ExpandedExpression>, Box<ExpandedExpression>),
    BitXor(Box<ExpandedExpression>, Box<ExpandedExpression>),
    BitNot(Box<ExpandedExpression>),

    IfThenElse(
        Box<ExpandedExpression>,
        Box<ExpandedExpression>,
        Box<ExpandedExpression>,
    ),
    Block(Box<ExpandedBlockExpr>),
    CondMatch(Box<DesugaredCondMatch>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesugaredCondMatch {
    /// The label that the condmatch value gets i.e. `__cmatch0`, `__cmatch1` etc.
    /// This label must be unique to the scope of the function in which it exists
    label: usize,
    /// All of its cases, which may or may not contain a default arm.
    arms: Vec<ExpandedExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedBlockExpr {
    /// The label that the block expression has its value assigned to
    /// i.e. `__block0`, `__block1` etc.
    /// This label must be unique to the scope of the function in which it exists
    pub label: String,
    pub statements: Vec<ExpandedStatement>,
    pub last: ExpandedExpression,
}
impl ExpandedBlockExpr {
    pub fn insert_return_unit(
        statements: Vec<ExpandedStatement>,
    ) -> (Vec<ExpandedStatement>, ExpandedExpression) {
        match statements.last() {
            Some(ExpandedStatement::Return(expr)) => {
                (statements[..(statements.len() - 1)].to_vec(), expr.clone())
            }
            _ => (statements, ExpandedExpression::Unit),
        }
    }

    pub fn new(label: impl Into<String>, statement_block: Vec<ExpandedStatement>) -> Self {
        let (statements, last) = Self::insert_return_unit(statement_block);
        Self {
            label: label.into(),
            statements,
            last,
        }
    }
}

impl From<Literal> for ExpandedExpression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

/// An assignment to a simple, totally unpacked variable.
#[derive(Debug, PartialEq, Clone)]
pub struct SimpleInitialisation {
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: ExpandedExpression,
}

impl SimpleInitialisation {
    pub fn new(typ: Option<Type>, assignee: impl Into<String>, value: ExpandedExpression) -> Self {
        Self {
            typ,
            assignee: assignee.into(),
            value,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ExpandedInitialisation {
    pub temporary: SimpleInitialisation,
    pub unpacked_assignments: Vec<ExpandedInitialisation>,
}

impl ExpandedInitialisation {
    pub(crate) fn new(
        temporary: SimpleInitialisation,
        unpacked_assignments: Vec<ExpandedInitialisation>,
    ) -> ExpandedInitialisation {
        Self {
            temporary,
            unpacked_assignments,
        }
    }

    pub fn simple(typ: Option<Type>, assignee: String, value: ExpandedExpression) -> Self {
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

impl From<SimpleInitialisation> for ExpandedInitialisation {
    fn from(value: SimpleInitialisation) -> Self {
        Self {
            temporary: value,
            unpacked_assignments: vec![],
        }
    }
}

impl From<SimpleInitialisation> for ExpandedStatement {
    fn from(value: SimpleInitialisation) -> Self {
        Self::Initialisation(value.into())
    }
}

impl Statement {}
