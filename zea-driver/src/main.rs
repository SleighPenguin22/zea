// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use std::process::exit;
use zea_ast::visualisation::IndentPrint;
use zea_ast::zea::visitors::altering::AcceptsAssignmentSimplifier;
use zea_parser::parse_module;

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let (module, _generator) = match parse_module(&src) {
        Ok((module, generator)) => {
            // println!("parsed {}", module.indent_print(0));
            (module, generator)
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };

    println!(
        "{}",
        module.has_assignments_unpacked() && module.has_assignments_flattened()
    );
    println!("after expansions:\n{}", module.indent_print(0));
}
