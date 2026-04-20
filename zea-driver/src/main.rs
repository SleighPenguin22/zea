// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use std::process::exit;
use zea_ast::visualisation::IndentPrint;
use zea_ast::zea::visitors::altering::{BareNodeLabeler, Relabel};
use zea_parser::ModuleParser;

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let mut node_labeler = BareNodeLabeler::new();
    let p = ModuleParser::new();
    let module = p.parse(&src);
    let mut module = match module {
        Ok(module) => {
            println!("parsed {}", module.indent_print(0));
            module
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    module.give_unique_ids(&mut node_labeler);

    let (module, generator) = module.expand_blocks(node_labeler);
    let (module, _generator) = module.simplify_assignments(generator);
    println!("after expansions:\n{}", module.indent_print(0));
}
