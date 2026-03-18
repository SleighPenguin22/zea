use std::fs::read_to_string;
use zea_parser::parse;
#[test]
fn parses_basic_main() {
    let src = read_to_string("tests/dummy_programs/basic_main.zea").unwrap();
    parse(&src).unwrap();
}

#[test]
fn parses_basic_main_with_globs() {
    let src = read_to_string("tests/dummy_programs/basic_main_with_globs.zea").unwrap();
    parse(&src).unwrap();
}
