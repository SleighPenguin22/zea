// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use std::process::exit;
use zea_ast::visualisation::IndentPrint;
use zea_parser::{desugar, parse_module};

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let (module, generator) = match parse_module(&src) {
        Ok((module, generator)) => {
            println!("parsed {}", module.indent_print(0));
            (module, generator)
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };

    let (module, _generator) = desugar(module, generator);
    println!("after expansions:\n{}", module.indent_print(0));
}
