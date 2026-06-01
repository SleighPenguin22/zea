// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use std::fs::read_to_string;
use zea_ast::visualisation::IndentPrint;
use zea_parser::parse_module;

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let (mut module, generator) = parse_module(&src);

    println!("after expansions:\n{}", module.indent_print(0));
    //
    module.simplify_assignments_after(generator);
    //
    println!("after expansions:\n{}", module.indent_print(0));
}
