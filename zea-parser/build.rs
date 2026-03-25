fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("./")
        .use_colors_if_tty()
        .set_out_dir("src/")
        .process()
        .unwrap();
}
