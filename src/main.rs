#![allow(unused)]
use crate::ast::{
    AssignmentPattern, Expression, Function, Initialisation, Literal, Statement, Type,
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
    let assigna = Statement::Initialisation(Initialisation {
        typ: Some(Type::Basic("I32".to_string())),
        assignee: AssignmentPattern::Identifier("a".to_string()),
        value: Expression::Literal(Literal::Integer(4)),
    });
    let return3 = Statement::Return(Expression::Ident("a".to_string()));
    let main = Function {
        name: "main".to_string(),
        args: vec![],
        returns: Type::Basic("I32".to_string()),
        body: vec![],
    };

    let expr = Expression::LogXor(
        Box::new(Expression::LogNot(Box::new(Expression::Sub(
            Box::new(Expression::Literal(Literal::String("bob".to_string()))),
            Box::new(Expression::Literal(Literal::Boolean(true))),
        )))),
        Box::new(Expression::Add(
            Box::new(Expression::Literal(Literal::Integer(3))),
            Box::new(Expression::Literal(Literal::Float(3.14))),
        )),
    );

    visualize_parse_tree::graphify(assigna, "pt.png")
}

#[cfg(not(feature = "visualisation"))]
fn main() {}
