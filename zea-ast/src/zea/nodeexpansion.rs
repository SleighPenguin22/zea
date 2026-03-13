#![allow(unused)]

use crate::zea::{AssignmentPattern, Function, FunctionCall, Module};
use crate::zea::{ExpandedBlockExpr, ExpandedInitialisation};
use crate::zea::{Expression, ExpressionKind};
use crate::zea::{Initialisation, StatementBlock, TypedIdentifier};
use crate::zea::{Statement, StatementKind};
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}

pub trait AcceptsNodeExpander {
    fn accept(&mut self, node_expander: &mut NodeExpander);
    fn is_expanded(&self) -> bool;
}

impl AcceptsNodeExpander for Statement {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        match &mut self.kind {
            StatementKind::Block(b) => {
                self.kind = StatementKind::ExpandedBlock(node_expander.expand_block(b));
            }
            StatementKind::Initialisation(i) => i.value.accept(node_expander),
            StatementKind::Reassignment(r) => r.value.accept(node_expander),
            StatementKind::FunctionCall(call) => call.accept(node_expander),
            StatementKind::Return(expr) => expr.accept(node_expander),
            StatementKind::BlockTail(expr) => expr.accept(node_expander),
            StatementKind::ExpandedInitialisation(init) => init.accept(node_expander),
            StatementKind::SimpleInitialisation(sinit) => sinit.value.accept(node_expander),
            StatementKind::ExpandedBlock(_) => {}
        }
        // self.accept(node_expander)
    }
    fn is_expanded(&self) -> bool {
        match &self.kind {
            StatementKind::Block(_) => false,

            StatementKind::Initialisation(i) => i.value.is_expanded(),
            StatementKind::Reassignment(r) => r.value.is_expanded(),
            StatementKind::FunctionCall(call) => call.is_expanded(),
            StatementKind::Return(expr) => expr.is_expanded(),
            StatementKind::BlockTail(expr) => expr.is_expanded(),
            StatementKind::ExpandedInitialisation(init) => init.is_expanded(),
            StatementKind::SimpleInitialisation(sinit) => sinit.value.is_expanded(),
            StatementKind::ExpandedBlock(b) => b.is_expanded(),
        }
    }
}

impl AcceptsNodeExpander for Expression {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        match &mut self.kind {
            ExpressionKind::Block(block) => {
                self.kind =
                    ExpressionKind::ExpandedBlock(Box::new(node_expander.expand_block(block)));
            }
            ExpressionKind::FuncCall(call) => call.accept(node_expander),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.accept(node_expander);
                rhs.accept(node_expander)
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.accept(node_expander),
            ExpressionKind::ExpandedBlock(block) => {}
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::Ident(_) => {}
        }
    }
    fn is_expanded(&self) -> bool {
        match &self.kind {
            ExpressionKind::Block(block) => false,
            ExpressionKind::FuncCall(call) => call.is_expanded(),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => lhs.is_expanded() && rhs.is_expanded(),
            ExpressionKind::UnOpExpr(_, arg) => arg.is_expanded(),
            ExpressionKind::ExpandedBlock(block) => block.is_expanded(),
            _ => true,
        }
    }
}

impl AcceptsNodeExpander for ExpandedInitialisation {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        self.temporary.value.accept(node_expander);
        for assign in &mut self.unpacked_assignments {
            assign.accept(node_expander);
        }
    }
    fn is_expanded(&self) -> bool {
        self.unpacked_assignments.iter().all(|a| a.is_expanded())
    }
}

impl AcceptsNodeExpander for FunctionCall {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        for arg in self.args.iter_mut() {
            arg.accept(node_expander)
        }
    }
    fn is_expanded(&self) -> bool {
        self.args.iter().all(|e| e.is_expanded())
    }
}

impl AcceptsNodeExpander for Function {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        self.body.accept(node_expander)
    }

    fn is_expanded(&self) -> bool {
        self.body.is_expanded()
    }
}

impl AcceptsNodeExpander for StatementBlock {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        for stmt in self.statements.iter_mut() {
            stmt.accept(node_expander)
        }
    }
    fn is_expanded(&self) -> bool {
        self.statements.iter().all(|s| s.is_expanded())
    }
}

impl AcceptsNodeExpander for ExpandedBlockExpr {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        self.last.accept(node_expander);
        for stmt in self.statements.iter_mut() {
            stmt.accept(node_expander);
        }
    }
    fn is_expanded(&self) -> bool {
        self.last.is_expanded() && self.statements.iter().all(|s| s.is_expanded())
    }
}

impl AcceptsNodeExpander for Module {
    fn accept(&mut self, node_expander: &mut NodeExpander) {
        for func in self.functions.iter_mut() {
            func.accept(node_expander)
        }
    }

    fn is_expanded(&self) -> bool {
        self.functions.iter().all(|f| f.is_expanded())
    }
}

impl Module {
    pub fn expand_blocks(mut self, node_expander: &mut NodeExpander) -> Module {
        self.accept(node_expander);
        self
    }
}
#[derive(Default)]
pub struct NodeExpander {
    labeler: usize,
}

/// Tranform some node into a given variant, and label it.

impl NodeExpander {
    pub fn new() -> Self {
        Self::default()
    }

    fn label(&mut self) -> usize {
        let label = self.labeler;
        self.labeler += 1;
        label
    }

    pub fn expand_block(&mut self, block: &StatementBlock) -> ExpandedBlockExpr {
        let (statements, last) = match block.statements.last() {
            Some(Statement {
                kind: StatementKind::BlockTail(_),
                ..
            }) => {
                let tail = block.statements.last().cloned().unwrap();
                let StatementKind::BlockTail(e) = tail.kind else {
                    unreachable!()
                };
                (block.statements.clone(), e)
            }
            _ => (block.statements.clone(), Expression::unit(self.label())),
        };

        ExpandedBlockExpr {
            id: self.label(),
            statements,
            last,
        }
    }

    pub fn expand_statement(&mut self, statement: Statement) -> Statement {
        let kind = match statement.kind {
            StatementKind::Block(b) => StatementKind::ExpandedBlock(self.expand_block(&b)),
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
                ExpressionKind::ExpandedBlock(Box::new(self.expand_block(&block)))
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
    use crate::zea::nodeexpansion::{AcceptsNodeExpander, NodeExpander};
    use crate::zea::{
        Function, FunctionCall, Module, Statement, StatementBlock, StatementKind, Type,
    };

    #[test]
    fn test_expand_block() {
        let mut node_expander = NodeExpander::new();
        let mut ast = Module::default();
        ast.functions.push(Function {
            id: 0,
            name: "main".to_string(),
            args: vec![],
            returns: Type::Basic("Int".to_string()),
            body: StatementBlock {
                id: 0,
                statements: vec![
                    Statement {
                        id: 0,
                        kind: StatementKind::Block(StatementBlock {
                            id: 0,
                            statements: vec![Statement {
                                id: 0,
                                kind: StatementKind::FunctionCall(FunctionCall {
                                    id: 0,
                                    name: "main".to_string(),
                                    args: vec![],
                                }),
                            }],
                        }),
                    },
                    Statement {
                        id: 0,
                        kind: StatementKind::Block(StatementBlock {
                            id: 0,
                            statements: vec![
                                Statement {
                                    id: 0,
                                    kind: StatementKind::FunctionCall(FunctionCall {
                                        id: 0,
                                        name: "main".to_string(),
                                        args: vec![],
                                    }),
                                },
                                Statement {
                                    id: 0,
                                    kind: StatementKind::Block(StatementBlock {
                                        id: 0,
                                        statements: vec![Statement {
                                            id: 0,
                                            kind: StatementKind::FunctionCall(FunctionCall {
                                                id: 0,
                                                name: "main".to_string(),
                                                args: vec![],
                                            }),
                                        },Statement {
                                            id: 0,
                                            kind: StatementKind::Block(StatementBlock {
                                                id: 0,
                                                statements: vec![Statement {
                                                    id: 0,
                                                    kind: StatementKind::FunctionCall(FunctionCall {
                                                        id: 0,
                                                        name: "main".to_string(),
                                                        args: vec![],
                                                    }),
                                                },Statement {
                                                    id: 0,
                                                    kind: StatementKind::Block(StatementBlock {
                                                        id: 0,
                                                        statements: vec![Statement {
                                                            id: 0,
                                                            kind: StatementKind::FunctionCall(FunctionCall {
                                                                id: 0,
                                                                name: "main".to_string(),
                                                                args: vec![],
                                                            }),
                                                        }],
                                                    }),
                                                },Statement {
                                                    id: 0,
                                                    kind: StatementKind::Block(StatementBlock {
                                                        id: 0,
                                                        statements: vec![Statement {
                                                            id: 0,
                                                            kind: StatementKind::FunctionCall(FunctionCall {
                                                                id: 0,
                                                                name: "main".to_string(),
                                                                args: vec![],
                                                            }),
                                                        }],
                                                    }),
                                                },Statement {
                                                    id: 0,
                                                    kind: StatementKind::Block(StatementBlock {
                                                        id: 0,
                                                        statements: vec![Statement {
                                                            id: 0,
                                                            kind: StatementKind::FunctionCall(FunctionCall {
                                                                id: 0,
                                                                name: "main".to_string(),
                                                                args: vec![],
                                                            }),
                                                        },Statement {
                                                            id: 0,
                                                            kind: StatementKind::Block(StatementBlock {
                                                                id: 0,
                                                                statements: vec![Statement {
                                                                    id: 0,
                                                                    kind: StatementKind::FunctionCall(FunctionCall {
                                                                        id: 0,
                                                                        name: "main".to_string(),
                                                                        args: vec![],
                                                                    }),
                                                                }],
                                                            }),
                                                        }],
                                                    }),
                                                }],
                                            }),
                                        }],
                                    }),
                                },
                            ],
                        }),
                    },
                ],
            },
        });

        let ast = ast.expand_blocks(&mut node_expander);
        eprintln!("{:?}", ast.functions[0]);
        assert!(ast.is_expanded())
    }
}
