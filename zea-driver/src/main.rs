// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use zea_ast::PrettyAST;
use std::fs::read_to_string;
use std::process::exit;
use zea_ast::zea::NodeExpander;
use zea_parser::{ModuleParser, NodeIdGenerator};

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let mut generator = NodeIdGenerator::new();
    let p = ModuleParser::new();
    let module = p.parse(&mut generator, &src);
    let module = match module {
        Ok(module) => {
            println!("parsed {}", module.pretty_print(0));
            module
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    
    let mut node_expander = NodeExpander::new();
    let module = module.expand_blocks(&mut node_expander);
    let module = module.simplify_assignments(&mut node_expander);
    println!("after expansions:\n{}", module.pretty_print(0));
    
    
}
