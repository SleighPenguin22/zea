// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use log::{error, info, trace};
use std::{fs::read_to_string, path::Path, process::exit as pexit};
use zea_config::CompilerConfig;
use zea_ipr::{visualisation::IndentPrint, zea::typecheck_module};
use zea_parser::parse_module;

fn exit(code: i32) -> ! {
    error!("exiting...");
    pexit(code)
}

fn read_to_string_wrapper(p: &Path) -> String {
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
    let config = CompilerConfig::parse_args();
    colog::basic_builder()
        .filter_level(config.log_level())
        .init();
    let src = read_to_string_wrapper(config.path());
    let (mut module, generator) = parse_module(&src);

    trace!("before expansions:\n{}", module.indent_print(0));
    module.simplify_assignments_after(generator);
    let (mut module, scopes) = module.scope_idents();
    info!("commencing typechecking...");
    let tinfo = typecheck_module(&mut module);
    info!("finished typechecking");
    if config.print_mir() {
        info!("after expansions:\n{}", module.indent_print(0));
    }

    info!("lowering into THR...");
    let lowered = zea_ipr::zea::lower_module(module, tinfo, scopes);
    info!("Typed Highlevel Representation:\n{:?}", lowered);
}
