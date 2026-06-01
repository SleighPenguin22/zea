#[cfg(feature = "lalrpop_parser")]
fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("./")
        .use_colors_if_tty()
        .set_out_dir("src/")
        .process()
        .unwrap();
}

#[cfg(not(feature = "lalrpop_parser"))]
fn main() {}
