use clap::Parser;
use std::fs::read_to_string;
use std::process::exit;
// use zea_parser::parse;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub filename: String,
}

fn main() {
    let src = read_to_string("zea-driver/test.zea").unwrap();
    // let module = parse(&src);
    // let module = match module {
    //     Ok((module, errs)) => {
    //         for err in errs {
    //             eprintln!("{err}\n++++++++++++++++++++++++++++++++++++++++++++++++");
    //         }
    //         module },
    //     Err(e) => {
    //         eprintln!("{e}");
    //         exit(1)
    //     }
    // };
    // eprint!("{module:?}");
}
