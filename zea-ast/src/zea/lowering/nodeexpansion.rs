#![allow(unused)]

use crate::zea::expression::Expression;
use crate::zea::lowering::{
    ExpandedBlockExpr, ExpandedExpression, ExpandedInitialisation, ExpandedStatement,
};
use crate::zea::patterns::AssignmentPattern;
use crate::zea::statement::Statement;
use crate::zea::{Function, Initialisation, StatementBlock, Type, TypedIdentifier};
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}
#[derive(Default)]
pub struct NodeExpander {
    block_label: usize,
    cond_match_label: usize,
    pattern_match_label: usize,
    named_tuple_cache: HashSet<TupleWithNamedMembers>,
}

impl NodeExpander {
    const BLOCK_LABEL_PREFIX: &str = "__block";
    const CONDMATCH_LABEL_PREFIX: &str = "__condmatch";
    const PATMATCH_LABEL_PREFIX: &str = "__patmatch";
    const TUPLE_LABEL_PREFIX: &str = "tuple";
    pub fn new() -> Self {
        Self::default()
    }

    fn gen_block_label(&mut self) -> String {
        let label = Self::BLOCK_LABEL_PREFIX.to_string() + &self.block_label.to_string();
        self.block_label += 1;
        label
    }
    fn gen_condmatch_label(&mut self) -> String {
        let label = Self::CONDMATCH_LABEL_PREFIX.to_string() + &self.block_label.to_string();
        self.cond_match_label += 1;
        label
    }
    fn gen_patmatch_label(&mut self) -> String {
        let label = Self::PATMATCH_LABEL_PREFIX.to_string() + &self.block_label.to_string();
        self.pattern_match_label += 1;
        label
    }

    pub fn expand_block(&mut self, block: StatementBlock) -> ExpandedBlockExpr {
        let block = block
            .into_iter()
            .map(|stmt| self.expand_statement(stmt))
            .collect();
        let label = self.gen_block_label();
        ExpandedBlockExpr::new(label, block)
    }

    pub fn expand_statement(&mut self, statement: Statement) -> ExpandedStatement {
        match statement {
            Statement::Block(b) => ExpandedStatement::LoweredBlock(self.expand_block(b)),
            Statement::Initialisation(assignment) => {
                ExpandedStatement::Initialisation(self.expand_assignment(assignment))
            }
            Statement::Return(expr) => ExpandedStatement::Return(self.expand_expression(expr)),
            _ => unimplemented!("lowering of remaining statements variants"),
        }
    }

    pub fn expand_assignment(&mut self, assignment: Initialisation) -> ExpandedInitialisation {
        match assignment.assignee {
            AssignmentPattern::Identifier(assignee) => ExpandedInitialisation::simple(
                assignment.typ,
                assignee,
                self.expand_expression(assignment.value),
            ),
            _ => unimplemented!("lowering of tuple unpacking assignments"),
        }
    }

    pub fn expand_expression(&mut self, expression: Expression) -> ExpandedExpression {
        match expression {
            Expression::Block(block) => {
                ExpandedExpression::Block(Box::new(self.expand_block(block)))
            }
            Expression::Literal(l) => ExpandedExpression::Literal(l),
            Expression::Ident(i) => ExpandedExpression::Ident(i),
            _ => unimplemented!("lower remaining expressions"),
        }
    }
}

#[derive(Debug)]
pub struct LoweredFunction {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: Vec<ExpandedStatement>,
}

pub fn lower_function(
    function: Function,
    module_named_tuple_cache: &mut ModuleNamedTupleCache,
) -> LoweredFunction {
    let mut label_generator = NodeExpander::new();
    let body: Vec<ExpandedStatement> = function
        .body
        .into_iter()
        .map(|stmt| label_generator.expand_statement(stmt))
        .collect();

    LoweredFunction {
        name: function.name,
        args: function.args,
        returns: function.returns,
        body,
    }
}

#[cfg(test)]
mod tests {
    use crate::zea::expression::{Expression, Literal};
    use crate::zea::lowering::nodeexpansion::{lower_function, ModuleNamedTupleCache};
    use crate::zea::patterns::AssignmentPattern;
    use crate::zea::statement::Statement;
    use crate::zea::test_utils::types::int_type;
    use crate::zea::{Function, Initialisation, StatementBlock};

    #[test]
    fn lower_basic_main() {
        let assigna = Statement::Initialisation(Initialisation {
            id: 0,
            typ: Some(int_type()),
            assignee: AssignmentPattern::Identifier("a".to_string()),
            value: Expression::Literal(Literal::Integer(4)),
        });
        let return3 = Statement::Return(Expression::Ident("a".to_string()));
        let main = Function {
            id: 0,
            name: "main".to_string(),
            args: vec![],
            returns: int_type(),
            body: StatementBlock {
                id: 0,
                stmts: vec![assigna, return3],
            },
        };

        // fn main() -> int {
        //  a: int = 4;
        // return a;
        // }

        let mut cache = ModuleNamedTupleCache::new();

        let main = dbg!(&lower_function(main, &mut cache));
    }
}
