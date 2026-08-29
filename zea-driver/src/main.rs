// #[derive(Parser, Debug)]
// #[command(version, about)]
// pub struct Args {
//     #[arg(short, long)]
//     pub filename: String,
// }

use log::{error, info, trace};
use std::{
    fs::{File, read_to_string},
    io::Write,
    path::{Path, PathBuf},
    process::exit as pexit,
};
use tempfile::NamedTempFile;
use zea_codegen::THRtoQBE;
use zea_common::CompilerConfig;
use zea_ipr::ast::thr::THRModule;
use zea_ipr::typecheck_module;
use zea_ipr::visualisation::IndentPrint;
use zea_parser::parse_module;

fn out_path(ccfg: &CompilerConfig, module: &THRModule) -> PathBuf {
    if let Some(file) = ccfg.out_file().cloned() {
        file
    } else {
        PathBuf::from(format!("{}.out", module.name))
    }
}
fn invoke_asm(ccfg: &CompilerConfig, module: &THRModule, asm_path: &Path) -> PathBuf {
    let out_path = out_path(ccfg, module);
    let status = std::process::Command::new("gcc")
        .arg(asm_path)
        .arg("-o")
        .arg(&out_path)
        .arg("-Wall")
        .status()
        .unwrap();

    if !status.success()
        && let Some(code) = status.code()
    {
        error!("GCC returned error code {code}, exiting...");
        exit(code)
    }
    trace!("saved compiled binary to {}", out_path.display());
    out_path
}
fn invoke_qbe(ccfg: &CompilerConfig, module: &THRModule, qbe_path: &Path) -> PathBuf {
    let path = if let Some(path) = ccfg.asm_file().cloned() {
        info!("saving assembly to {}", path.display());
        path
    } else {
        let asm_temp = NamedTempFile::with_suffix_in(format!("_{}.s", module.name), "./").unwrap();
        let (_, asm_temp) = asm_temp.keep().unwrap();
        asm_temp
    };
    let status = std::process::Command::new("qbe")
        .arg(qbe_path)
        .arg("-o")
        .arg(&path)
        .status()
        .unwrap();

    if !status.success()
        && let Some(code) = status.code()
    {
        error!("QBE returned error code {code}, exiting...");
        exit(code)
    }
    trace!("saved compiled QBE module assembly to {}", path.display());
    path
}
fn write_qbe_il(ccfg: &CompilerConfig, module: &THRModule, il: &str) -> PathBuf {
    let (mut f, path) = if let Some(path) = ccfg.qbe_file().cloned() {
        info!("saving QBE IL to {}", path.display());
        let f = File::create(path.as_path()).unwrap();
        (f, path)
    } else {
        let temp = NamedTempFile::with_suffix_in(format!("_{}.qbe", module.name), "./").unwrap();
        temp.keep().unwrap()
    };
    f.write_all(il.as_bytes()).unwrap();
    path
}

fn cleanup_temp_files(ccfg: &CompilerConfig, qbe_il: &Path, asm: &Path) {
    if ccfg.asm_file().is_none() {
        trace!("cleaning up {}", asm.display());
        std::fs::remove_file(asm).unwrap()
    }

    if ccfg.qbe_file().is_none() {
        trace!("cleaning up {}", qbe_il.display());
        std::fs::remove_file(qbe_il).unwrap()
    }
}
fn exit(code: i32) -> ! {
    error!("exiting...");
    pexit(code)
}

fn read_to_string_wrapper(p: &Path) -> String {
    use std::io::ErrorKind;
    let p_disp = p.display();
    trace!("attempting read of file `{p_disp}`");
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
    let ccfg = CompilerConfig::parse_args();
    colog::basic_builder().filter_level(ccfg.log_level()).init();
    let src = read_to_string_wrapper(ccfg.path());
    let (mut module, generator) = parse_module(&src);

    let a = module.simplify_assignments_after(generator);
    module.insert_implicit_main_return(a);

    let (mut module, scopes) = module.scope_idents_diverging();
    info!("commencing typechecking...");
    let tinfo = typecheck_module(&mut module);
    info!("finished typechecking");
    if ccfg.print_ipr() {
        info!("after expansions:\n{}", module.indent_print(0));
        info!("module Debug Print: {module:?}");
    }

    info!("lowering into THR...");
    let lowered = zea_ipr::ast::thr::lower_module(module, tinfo, scopes);
    if ccfg.print_thr() {
        info!("Typed Highlevel Representation:\n{:?}", lowered);
    }

    let mut codegen = THRtoQBE::new(&lowered);
    let qbe = codegen.lower();
    let il = format!("{qbe}");
    if ccfg.print_qbe_il() {
        info!("Generated QBE IL:\n{il}");
    }

    let qbe_path = write_qbe_il(&ccfg, &lowered, &il);

    let asm_path = invoke_qbe(&ccfg, &lowered, &qbe_path);
    let _out_path = invoke_asm(&ccfg, &lowered, &asm_path);

    info!(
        "saved compiled binary to {}",
        _out_path.canonicalize().unwrap().display()
    );
    cleanup_temp_files(&ccfg, &qbe_path, &asm_path);
}
