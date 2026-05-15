// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use std::process::exit;
use zea_ast::visualisation::IndentPrint;
use zea_ast::zea::{BlockExpander, NodeLabeler};
use zea_ast::zea::visitors::altering::AssignmentSimplifier;
use zea_ast::zea::visitors::Transfomer;
use zea_parser::parse_module;

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let (mut module, generator) = match parse_module(&src) {
        Ok((module, generator)) => {
            // println!("parsed {}", module.indent_print(0));
            (module, generator)
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    println!("after expansions:\n{}", module.indent_print(0));


    let mut block_expander: BlockExpander = generator.labeler_into();
    block_expander.visit_module(&mut module);
    let mut simplifier: AssignmentSimplifier = block_expander.labeler_into();
    simplifier.visit_module(&mut module);

    println!("after expansions:\n{}", module.indent_print(0));
}
