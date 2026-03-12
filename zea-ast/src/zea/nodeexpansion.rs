#![allow(unused)]

use crate::zea::patterns::AssignmentPattern;
use crate::zea::{Expression, ExpressionKind};
use crate::zea::{Initialisation, StatementBlock, TypedIdentifier};
use crate::zea::{Statement, StatementKind};
use std::collections::HashSet;
use crate::zea::statement::ExpandedBlockExpr;

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

    pub fn expand_block(&mut self, block: StatementBlock) -> ExpandedBlockExpr {
        let block: Vec<_> = block
            .into_iter()
            .map(|stmt| self.expand_statement(stmt))
            .collect();

        let (statements, last) = if let Some((last, init)) = block.split_last() {
            match last.kind {
                StatementKind::Return(ref expr) => (init.to_vec(), expr.clone()),
                _ => (
                    block,
                    Expression {
                        id: self.label(),
                        kind: ExpressionKind::Unit,
                    },
                ),
            }
        } else {
            (
                vec![],
                Expression {
                    id: self.label(),
                    kind: ExpressionKind::Unit,
                },
            )
        };

        ExpandedBlockExpr {
            id: self.label(),
            statements: statements,
            last,
        }
    }

    pub fn expand_statement(&mut self, statement: Statement) -> Statement {
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

    pub fn expand_assignment(&mut self, assignment: Initialisation) -> Expression {
        match assignment.assignee {
            AssignmentPattern::Identifier(assignee) => todo!(),
            _ => todo!("cannot yet expand tuple assignment\n{assignment:?}\n"),
        }
    }

    pub fn expand_expression(&mut self, expression: Expression) -> Expression {
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

    fn test_expand_block() {}
}
