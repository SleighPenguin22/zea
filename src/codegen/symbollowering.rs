#![allow(unused)]

use crate::ast::zea::expression::Expression;
use crate::ast::zea::patterns::AssignmentPattern;
use crate::ast::zea::statement::Statement;
use crate::ast::zea::{Function, Initialisation, StatementBlock, Type, TypedIdentifier};
use crate::lowering::{
    ExpandedBlockExpr, ExpandedExpression, ExpandedInitialisation, ExpandedStatement,
};
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}
#[derive(Default)]
pub struct DesugaredConstructLabelFactory {
    block_label: usize,
    cond_match_label: usize,
    pattern_match_label: usize,
    named_tuple_cache: HashSet<TupleWithNamedMembers>,
}

impl DesugaredConstructLabelFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name_block(&mut self, block: StatementBlock) -> ExpandedBlockExpr {
        let block = block
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        let label = "__block".to_string() + &self.block_label.to_string();
        self.block_label += 1;
        ExpandedBlockExpr::new(label, block)
    }

    pub fn lower_statement(&mut self, statement: Statement) -> ExpandedStatement {
        match statement {
            Statement::Block(b) => ExpandedStatement::LoweredBlock(self.name_block(b)),
            Statement::Initialisation(assignment) => {
                ExpandedStatement::Initialisation(self.lower_assignment(assignment))
            }
            Statement::Return(expr) => ExpandedStatement::Return(self.lower_expression(expr)),
            _ => unimplemented!("lowering of remaining statements variants"),
        }
    }

    pub fn lower_assignment(&mut self, assignment: Initialisation) -> ExpandedInitialisation {
        match assignment.assignee {
            AssignmentPattern::Identifier(assignee) => ExpandedInitialisation::simple(
                assignment.typ,
                assignee,
                self.lower_expression(assignment.value),
            ),
            _ => unimplemented!("lowering of tuple unpacking assignments"),
        }
    }

    pub fn lower_expression(&mut self, expression: Expression) -> ExpandedExpression {
        match expression {
            Expression::Block(block) => ExpandedExpression::Block(Box::new(self.name_block(block))),
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
    let mut label_generator = DesugaredConstructLabelFactory::new();
    let body: Vec<ExpandedStatement> = function
        .body
        .into_iter()
        .map(|stmt| label_generator.lower_statement(stmt))
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
    use crate::ast::zea::expression::{Expression, Literal};
    use crate::ast::zea::patterns::AssignmentPattern;
    use crate::ast::zea::statement::Statement;
    use crate::ast::zea::test_utils::types::int_type;
    use crate::ast::zea::{Function, Initialisation};
    use crate::codegen::symbollowering::{lower_function, ModuleNamedTupleCache};

    #[test]
    fn lower_basic_main() {
        let assigna = Statement::Initialisation(Initialisation {
            typ: Some(int_type()),
            assignee: AssignmentPattern::Identifier("a".to_string()),
            value: Expression::Literal(Literal::Integer(4)),
        });
        let return3 = Statement::Return(Expression::Ident("a".to_string()));
        let main = Function {
            name: "main".to_string(),
            args: vec![],
            returns: int_type(),
            body: vec![assigna, return3],
        };

        // fn main() -> int {
        //  a: int = 4;
        // return a;
        // }

        let mut cache = ModuleNamedTupleCache::new();

        let main = dbg!(&lower_function(main, &mut cache));
    }
}
