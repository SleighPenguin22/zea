use crate::ast::datatype::TupleSignature;
use crate::ast::expression::PatternMatchArm;
use crate::ast::{
    AssignmentPattern, Expression, Initialisation, StatementBlock, StructDefinition, Type,
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

#[derive(Debug, PartialEq, Clone)]
pub struct LoweredInitialisation {
    pub mutable: bool,
    pub typ: Type,
    pub assignee: String,
    pub value: Expression,
}
impl LoweredInitialisation {
    pub fn mutable(typ: Type, assignee: String, value: Expression) -> Self {
        Self {
            mutable: true,
            typ,
            assignee,
            value,
        }
    }
    pub fn constant(typ: Type, assignee: String, value: Expression) -> Self {
        Self {
            mutable: false,
            typ,
            assignee,
            value,
        }
    }
}

pub trait DesugarDestructuring {
    type Output;
    fn desugar_destructuring(self) -> LoweringResult<Self::Output>;
}

impl DesugarDestructuring for Initialisation {
    type Output = Vec<DesugaredInitialisation>;
    fn desugar_destructuring(self) -> LoweringResult<Self::Output> {
        Ok(match self.assignee {
            AssignmentPattern::Identifier(assignee) => {
                vec![DesugaredInitialisation {
                    temporary: LoweredInitialisation::constant(self.typ, assignee, self.value),
                    unpacked_assignments: vec![],
                }]
            }
            AssignmentPattern::Tuple(_tuple) => {
                unimplemented!("implement tuple pattern assignment destructuring blabla")
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredInitialisation {
    pub temporary: LoweredInitialisation,
    pub unpacked_assignments: Vec<LoweredInitialisation>,
}

pub trait DesugarMatchExpression {
    fn desugar_match_arm(arm: PatternMatchArm) -> LoweringResult<StatementBlock>;
    fn desugar_match_expression(&self) -> LoweringResult<StatementBlock> {
        todo!()
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
