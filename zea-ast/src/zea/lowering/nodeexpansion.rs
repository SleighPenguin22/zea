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
    label: usize,
    named_tuple_cache: HashSet<TupleWithNamedMembers>,
}

macro_rules! label_expr {
    ($variant:ident, $val:expr) => {{
        Expression {
            id: self.label(),
            kind: ExpressionKind::$variant($val),
        }
    }};
}

macro_rules! label_exexpr {
    ($s:ident, $variant:ident, $val:expr) => {{
        ExpandedExpression {
            id: $s.label(),
            kind: ExpandedExpressionKind::$variant($val),
        }
    }};
}
impl NodeExpander {
    const BLOCK_LABEL_PREFIX: &str = "__block";
    const CONDMATCH_LABEL_PREFIX: &str = "__condmatch";
    const PATMATCH_LABEL_PREFIX: &str = "__patmatch";
    const TUPLE_LABEL_PREFIX: &str = "tuple";

    pub fn new() -> Self {
        Self::default()
    }

    fn label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
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
            StatementKind::Block(b) => ExpandedStatement {
                id: self.label(),
                kind: ExpandedStatementKind::LoweredBlock(self.expand_block(b)),
            },
            StatementKind::Initialisation(assignment) => ExpandedStatement {
                id: self.label(),
                kind: ExpandedStatementKind::Initialisation(self.expand_assignment(assignment)),
            },
            StatementKind::Return(expr) => ExpandedStatement {
                id: self.label(),
                kind: ExpandedStatementKind::Return(self.expand_expression(expr)),
            },
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
            ExpressionKind::Block(block) => ExpandedExpression {
                id: self.label(),
                kind: ExpandedExpressionKind::Block(Box::new(self.expand_block(block))),
            },
            ExpressionKind::Ident(i) => label_exexpr!(self, Ident, i),
            ExpressionKind::IntegerLiteral(l) => ExpandedExpression {
                id: self.label(),
                kind: ExpandedExpressionKind::IntegerLiteral(l),
            },
            _ => todo!("cannot yet expand expression\n{expression:?}\n"),
        }
    }
}

#[cfg(test)]
mod tests {}
