// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use log::{error, info};
use std::fs::read_to_string;
use zea_ast::visualisation::IndentPrint;
use zea_parser::parse_module;

fn main() {
    colog::basic_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let src = read_to_string("zea-driver/test.zea").unwrap();
    let (mut module, generator) = parse_module(&src);

    info!("before expansions:\n{}", module.indent_print(0));
    //
    module.simplify_assignments_after(generator);

    // let mut type_checker = zea_ast::zea::ZeaTypeChecker::new();
    // match type_checker.check_module(&mut module) {
    //     Ok(_) => {}
    //     Err(e) => {
    //         error!("Type checking error: {e:?}")
    //     }
    // };
    // info!("after expansions:\n{}", module.indent_print(0));
}
