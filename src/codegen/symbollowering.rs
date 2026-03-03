#![allow(unused)]
use crate::ast::{
    AssignmentPattern, Expression, Function, Initialisation, Statement, StatementBlock, Type,
    TypedIdentifier,
};
use crate::lowering::{
    DesugaredBlockExpr, DesugaredInitialisation, LoweredExpression, LoweredStatement,
};
use std::collections::HashSet;

pub type ModuleNamedTupleCache = HashSet<TupleWithNamedMembers>;

pub struct TupleWithNamedMembers {
    members: Vec<TypedIdentifier>,
}

pub struct DesugaredConstructLabelFactory {
    block_label: usize,
    cond_match_label: usize,
    pattern_match_label: usize,
    named_tuple_cache: HashSet<TupleWithNamedMembers>,
}

impl Default for DesugaredConstructLabelFactory {
    fn default() -> Self {
        Self {
            block_label: 0,
            cond_match_label: 0,
            pattern_match_label: 0,
            named_tuple_cache: HashSet::new(),
        }
    }
}

impl DesugaredConstructLabelFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name_block(&mut self, block: StatementBlock) -> DesugaredBlockExpr {
        let block = block
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        let label = "__block".to_string() + &self.block_label.to_string();
        self.block_label += 1;
        DesugaredBlockExpr::new(label, block)
    }

    pub fn lower_statement(&mut self, statement: Statement) -> LoweredStatement {
        match statement {
            Statement::Block(b) => LoweredStatement::LoweredBlock(self.name_block(b)),
            Statement::Initialisation(assignment) => {
                LoweredStatement::Initialisation(self.lower_assignment(assignment))
            }
            Statement::Return(expr) => LoweredStatement::Return(self.lower_expression(expr)),
            _ => unimplemented!("lowering of remaining statements variants"),
        }
    }

    pub fn lower_assignment(&mut self, assignment: Initialisation) -> DesugaredInitialisation {
        match assignment.assignee {
            AssignmentPattern::Identifier(assignee) => DesugaredInitialisation::simple(
                assignment.typ,
                assignee,
                self.lower_expression(assignment.value),
            ),
            _ => unimplemented!("lowering of tuple unpacking assignments"),
        }
    }

    pub fn lower_expression(&mut self, expression: Expression) -> LoweredExpression {
        match expression {
            Expression::Block(block) => LoweredExpression::Block(Box::new(self.name_block(block))),
            Expression::Literal(l) => LoweredExpression::Literal(l),
            Expression::Ident(i) => LoweredExpression::Ident(i),
            _ => unimplemented!("lower remaining expressions"),
        }
    }
}

#[derive(Debug)]
pub struct LoweredFunction {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: Vec<LoweredStatement>,
}

pub fn lower_function(
    function: Function,
    module_named_tuple_cache: &mut ModuleNamedTupleCache,
) -> LoweredFunction {
    let mut label_generator = DesugaredConstructLabelFactory::new();
    let body: Vec<LoweredStatement> = function
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
    use crate::ast::test_utils::types::int_type;
    use crate::ast::{AssignmentPattern, Expression, Function, Initialisation, Literal, Statement};
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
