// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use clap::Parser;
use log::{error, info, trace, LevelFilter};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    process::exit as pexit,
};
use zea_ast::{visualisation::IndentPrint, zea::typecheck_module};
use zea_parser::parse_module;

#[derive(Parser)]
#[command(version, about)]
struct Config {
    path: PathBuf,

    #[arg(long = "loglevel", default_value_t = log::LevelFilter::Error)]
    log_level: LevelFilter,

    #[arg(long = "print-mir", default_value_t = false)]
    print_mir: bool,
}

fn exit(code: i32) -> ! {
    error!("exiting...");
    pexit(code)
}

fn read_to_string_wrapper(p: PathBuf) -> String {
    use std::io::ErrorKind;
    let p_disp = p.display();
    info!("attempting read of file `{p_disp}`");
    match p.canonicalize().and_then(|pc| read_to_string(&pc)) {
        Ok(s) => s,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    error!("file `{}` not found", p_disp);
                }
                ErrorKind::PermissionDenied => {
                    error!("you do not have adequate permissions to read `{}`", p_disp);
                }
                ErrorKind::IsADirectory => {
                    error!("the path `{}` is a directory", p_disp);
                }
                _ => todo!(),
            };
            exit(1);
        }
    }
}

fn main() {
    let config = Config::parse();
    colog::basic_builder().filter_level(config.log_level).init();
    let src = read_to_string_wrapper(config.path);
    let (mut module, generator) = parse_module(&src);

    trace!("before expansions:\n{}", module.indent_print(0));
    module.simplify_assignments_after(generator);
    let (mut module, _) = module.scope_idents();
    info!("commencing typechecking...");
    typecheck_module(&mut module);
    info!("finished typechecking");
    if config.print_mir {
        info!("after expansions:\n{}", module.indent_print(0));
    }
}
