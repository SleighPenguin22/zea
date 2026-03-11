#![allow(unused)]

use crate::zea::expression::{Expression, ExpressionKind};
use crate::zea::lowering::{
    ExpandedBlockExpr, ExpandedExpression, ExpandedExpressionKind, ExpandedInitialisation,
    ExpandedStatement, ExpandedStatementKind,
};
use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::{Statement, StatementKind};
use crate::zea::{Function, Initialisation, StatementBlock, Type, TypedIdentifier};
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}
#[derive(Default)]
pub struct NodeExpander {
    _labeler: usize,
    named_tuple_cache: HashSet<TupleWithNamedMembers>,
}

/// Tranform some node into a given variant, and label it.
macro_rules! label {
    (using $s:ident stmt $variant:ident with $val:expr) => {
        ExpandedStatement {
            id: $s.label(),
            kind: ExpandedStatementKind::$variant($val),
        }
    };
    (using $s:ident stmt $variant:ident) => {{
        ExpandedStatement {
            id: $s.label(),
            kind: ExpandedStatementKind::Return(label_exexpr!(self, Unit)),
        }
    }};
    (using $s:ident expr $variant:ident) => {{
        ExpandedExpression {
            id: $s.label(),
            kind: ExpandedExpressionKind::$variant,
        }
    }};
    (using $s:ident expr $variant:ident with $val:expr) => {{
        ExpandedExpression {
            id: $s.label(),
            kind: ExpandedExpressionKind::$variant($val),
        }
    }};
    (using $s:ident expr $variant:ident with $op:expr, $arg:expr) => {{
        ExpandedExpression {
            id: $s.label(),
            kind: ExpandedExpressionKind::$variant($op, $val),
        }
    }};
    (using $s:ident expr $variant:ident with $op:expr, $a:expr, $b:expr) => {{
        ExpandedExpression {
            id: $s.label(),
            kind: ExpandedExpressionKind::$variant($op, $a, $b),
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

    pub fn expand_block(&mut self, block: StatementBlock) -> ExpandedBlockExpr {
        let block: Vec<_> = block
            .into_iter()
            .map(|stmt| self.expand_statement(stmt))
            .collect();

        let (statements, last) = if let Some((last, init)) = block.split_last() {
            match last.kind {
                ExpandedStatementKind::Return(ref expr) => (init.to_vec(), expr.clone()),
                _ => (
                    block,
                    ExpandedExpression {
                        id: self.label(),
                        kind: ExpandedExpressionKind::Unit,
                    },
                ),
            }
        } else {
            (
                vec![],
                ExpandedExpression {
                    id: self.label(),
                    kind: ExpandedExpressionKind::Unit,
                },
            )
        };

        ExpandedBlockExpr {
            id: self.label(),
            statements: statements,
            last,
        }
    }

    pub fn expand_statement(&mut self, statement: Statement) -> ExpandedStatement {
        match statement.kind {
            StatementKind::Block(b) => label!(using self stmt Block with self.expand_block(b)),
            StatementKind::Initialisation(assignment) => {
                label!(using self stmt Initialisation with self.expand_assignment(assignment))
            }
            StatementKind::Return(expr) => {
                label!(using self stmt Return with self.expand_expression(expr))
            }
            _ => todo!("cannot yet expand statement\n{statement:?}\n"),
        }
    }

    pub fn expand_assignment(&mut self, assignment: Initialisation) -> ExpandedInitialisation {
        match assignment.assignee {
            AssignmentPattern::Identifier(assignee) => ExpandedInitialisation::simple(
                assignment.typ,
                assignee,
                self.expand_expression(assignment.value),
            ),
            _ => todo!("cannot yet expand tuple assignment\n{assignment:?}\n"),
        }
    }

    pub fn expand_expression(&mut self, expression: Expression) -> ExpandedExpression {
        match expression.kind {
            ExpressionKind::Block(block) => {
                label!(using self expr Block with Box::new(self.expand_block(block)))
            }
            ExpressionKind::Ident(i) => label!(using self expr Ident with i),
            ExpressionKind::IntegerLiteral(l) => label!(using self expr IntegerLiteral with l),
            _ => todo!("cannot yet expand expression\n{expression:?}\n"),
        }
    }
}

#[cfg(test)]
mod tests {

    fn test_expand_block() {

    }
}
