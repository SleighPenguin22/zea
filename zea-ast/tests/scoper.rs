use zea_ast::visualisation::IndentPrint;
use zea_parser::zepast;

#[test]
fn global_in_func_body() {
    let module = zepast::parse_module(include_str!("asts/global_in_func_body.zeast"));
}
