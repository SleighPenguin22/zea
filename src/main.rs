#![allow(unused)]

use crate::ast::{
    AssignmentPattern, Expression, Function, Initialisation, Module, Statement, TopLevelStatement,
    Type, TypedIdentifier,
};

mod analysis;
mod ast;
mod codegen;
mod driver;
mod lowering;
mod parser;
#[cfg(feature = "visualisation")]
pub mod visualize_parse_tree;

#[cfg(feature = "visualisation")]
fn main() {
    macro_rules! set {
        () => {{use std::collections::HashSet;HashSet::new()}};
        ($($e:expr),+) => {{
            use std::collections::HashSet;
            HashSet::from_iter(vec![$($e),+])
        }}
    }
    let assigna = Statement::Initialisation(Initialisation {
        typ: None,
        assignee: AssignmentPattern::Tuple(vec![
            AssignmentPattern::Identifier("x".to_string()),
            AssignmentPattern::Identifier("y".to_string()),
        ]),
        value: Expression::Ident("point".to_string()),
    });
    let return3 = Statement::Return(Expression::Ident("a".to_string()));

    let retexpr = Statement::Return(Expression::Unit);
    let square = Function {
        name: "square".to_string(),
        args: vec![TypedIdentifier::new(
            Type::Basic("I32".to_string()),
            "n".to_string(),
        )],
        returns: Type::Basic("I32".to_string()),
        body: vec![Statement::Return(Expression::Mul(
            Box::new(Expression::Ident("n".to_string())),
            Box::new(Expression::Ident("n".to_string())),
        ))],
    };
    let main2 = Function {
        name: "main2".to_string(),
        args: vec![],
        returns: Type::Basic("I32".to_string()),
        body: vec![assigna, return3],
    };
    let module = Module {
        imports: vec![],
        exports: vec![],
        symbols: set![
            TopLevelStatement::FuncDefinition(square),
            TopLevelStatement::FuncDefinition(main2)
        ],
    };

    match visualize_parse_tree::graphify(&module, "renders/pt.png") {
        Err(e) => eprintln!("could not form graph for AST: {e}"),
        Ok(()) => {}
    }
}

#[cfg(not(feature = "visualisation"))]
fn main() {}
