#![allow(unused)]

use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::{ExpandedBlockExpr, ExpandedInitialisation};
use crate::zea::{Expression, ExpressionKind};
use crate::zea::{Initialisation, StatementBlock, TypedIdentifier};
use crate::zea::{Statement, StatementKind};
use crate::VisitExpr;
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}
#[derive(Default)]
pub struct NodeExpander {
    _labeler: usize,
}

/// Tranform some node into a given variant, and label it.
macro_rules! label {
    (using $s:ident stmt $variant:ident with $val:expr) => {
        Statement {
            id: $s.label(),
            kind: StatementKind::$variant($val),
        }
    };
    (using $s:ident stmt $variant:ident) => {{
        Statement {
            id: $s.label(),
            kind: StatementKind::Return(label_exexpr!(self, Unit)),
        }
    }};
    (using $s:ident expr $variant:ident) => {{
        Expression {
            id: $s.label(),
            kind: ExpressionKind::$variant,
        }
    }};
    (using $s:ident expr $variant:ident with $val:expr) => {{
        Expression {
            id: $s.label(),
            kind: ExpressionKind::$variant($val),
        }
    }};
    (using $s:ident expr $variant:ident with $op:expr, $arg:expr) => {{
        Expression {
            id: $s.label(),
            kind: ExpressionKind::$variant($op, $val),
        }
    }};
    (using $s:ident expr $variant:ident with $op:expr, $a:expr, $b:expr) => {{
        Expression {
            id: $s.label(),
            kind: ExpressionKind::$variant($op, $a, $b),
        }
    }};
}
impl NodeExpander {
    pub fn new() -> Self {
        Self::default()
    }

    fn label(&mut self) -> usize {
        let label = self._labeler;
        self._labeler += 1;
        label
    }

    pub fn expand_block(&mut self, mut block: StatementBlock) -> ExpandedBlockExpr {
        let (statements, last) = match block.statements.last() {
            Some(Statement {
                kind: StatementKind::BlockTail(_),
                ..
            }) => {
                let tail = block.statements.pop().unwrap();
                let StatementKind::BlockTail(e) = tail.kind else {
                    unreachable!()
                };
                (block.statements, e)
            }
            _ => (block.statements, Expression::unit(self.label())),
        };

        ExpandedBlockExpr {
            id: self.label(),
            statements: statements,
            last,
        }
    }

    pub fn expand_statement(&mut self, statement: Statement) -> Statement {
        let kind = match statement.kind {
            StatementKind::Block(b) => StatementKind::ExpandedBlock(self.expand_block(b)),
            StatementKind::Initialisation(assignment) => {
                StatementKind::ExpandedInitialisation(self.expand_assignment(assignment))
            }
            _ => return statement,
        };
        Statement {
            id: self.label(),
            kind,
        }
    }

    pub fn expand_assignment(&mut self, assignment: Initialisation) -> ExpandedInitialisation {
        match &assignment.assignee {
            AssignmentPattern::Identifier(assignee) => todo!(),
            _ => todo!(),
        }
    }

    pub fn expand_expression(&mut self, expression: Expression) -> Expression {
        let kind = match expression.kind {
            ExpressionKind::Block(block) => {
                ExpressionKind::ExpandedBlock(Box::new(self.expand_block(block)))
            }
            ref _other => return expression,
        };
        Expression {
            id: self.label(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    fn test_expand_block() {}
}
