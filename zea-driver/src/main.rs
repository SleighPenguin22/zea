// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use std::process::exit;
use zea_ast::visualisation::IndentPrint;
use zea_ast::zea::BlockExpander;
use zea_parser::{ModuleParser, NodeIdGenerator};

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let mut generator = NodeIdGenerator::new();
    let p = ModuleParser::new();
    let module = p.parse(&mut generator, &src);
    let module = match module {
        Ok(module) => {
            println!("parsed {}", module.indent_print(0));
            module
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };

    let node_expander = BlockExpander::new();
    let (module, generator) = module.expand_blocks(node_expander);
    let (module, _generator) = module.simplify_assignments(generator);
    println!("after expansions:\n{}", module.indent_print(0));
}
