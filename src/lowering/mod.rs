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
    pub typ: Option<Type>,
    pub assignee: String,
    pub value: Expression,
}
impl LoweredInitialisation {
    pub fn new(typ: Option<Type>, assignee: impl Into<String>, value: Expression) -> Self {
        Self {
            typ,
            assignee: assignee.into(),
            value,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct DesugaredInitialisation {
    pub temporary: LoweredInitialisation,
    pub unpacked_assignments: Vec<LoweredInitialisation>,
}

impl Initialisation {
    pub fn desugar_destructuring(self) -> LoweringResult<Vec<LoweredInitialisation>> {
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

impl From<DesugaredInitialisation> for Vec<LoweredInitialisation> {
    fn from(initialiser: DesugaredInitialisation) -> Self {
        let mut v = Vec::with_capacity(initialiser.unpacked_assignments.len() + 1);
        v.push(initialiser.temporary);
        v.extend(initialiser.unpacked_assignments);
        v
    }
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
