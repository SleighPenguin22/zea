#![allow(unused)]
use crate::ast::{Function, Statement, StatementBlock, Type, TypedIdentifier};
use crate::lowering::{DesugaredBlockExpr, LoweredExpression, LoweredStatement};
use std::collections::HashSet;

pub struct ModuleNamedTupleCache {
    named_tuples: HashSet<TupleWithNamedMembers>,
}

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
            _ => unimplemented!("lowering of remaining statements"),
        }
    }
}

pub struct LoweredFunction {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: Type,
    pub body: Vec<LoweredExpression>,
}

pub fn lower_function(
    function: Function,
    module_named_tuple_cache: &mut ModuleNamedTupleCache,
) -> LoweredFunction {
    let label_generator = DesugaredConstructLabelFactory::new();
    todo!()
}
