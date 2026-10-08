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
use zea_common::{CompilerConfig, CompilerError, ZeaError};
use zea_irs::ast::{
    ipr_walkers::{
        IPRTransfomer,
        transformers::{IdentifierScoper, scope_module},
    },
    thr::THRModule,
};
use zea_irs::visualisation::IndentPrint;
use zea_irs::{ZeaTypeChecker, typecheck_module};
use zea_parser::parse_module;

/// exit with an ICE message when an operation fails
fn exit_on_ice<T>(res: Result<T, CompilerError>) -> T {
    match res {
        Ok(t) => t,
        Err(e) => {
            error!("{}", e.pretty());
            exit(1);
        }
    }
}
fn exit_on_user_error<'s, 'c, T, C, E: ZeaError<C>>(res: Result<T, E>, ctx: &'c C) -> T {
    match res {
        Ok(t) => t,
        Err(e) => {
            let msg = e.zea_error_format(ctx);
            error!("{}", msg);
            exit(1);
        }
    }
}

fn out_path(ccfg: &CompilerConfig, module: &THRModule) -> PathBuf {
    if let Some(file) = ccfg.out_file().cloned() {
        file
    } else {
        PathBuf::from(format!("{}.out", module.name))
    }
}

fn temp_file_in_out_dir(ccfg: &CompilerConfig, suffix: String) -> (File, PathBuf) {
    let temp_file = NamedTempFile::with_suffix_in(suffix, ccfg_out_dir(ccfg)).unwrap();
    temp_file.keep().unwrap()
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
        let (_file, asm_path) = temp_file_in_out_dir(ccfg, format!("_{}.S", module.name));
        asm_path
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
fn write_qbe_il(ccfg: &CompilerConfig, module: &THRModule, il_str: &str) -> PathBuf {
    let (mut f, path) = if let Some(path) = ccfg.qbe_file().cloned() {
        info!("saving QBE IL to {}", path.display());
        let f = File::create(path.as_path()).unwrap();
        (f, path)
    } else {
        temp_file_in_out_dir(ccfg, format!("_{}.qbe", module.name))
    };
    f.write_all(il_str.as_bytes()).unwrap();
    path
}

fn cleanup_temp_files(ccfg: &CompilerConfig, qbe_il_path: &Path, asm_path: &Path) {
    if ccfg.keep_files() {
        return;
    }
    if ccfg.asm_file().is_none() {
        trace!("cleaning up {}", asm_path.display());
        std::fs::remove_file(asm_path).unwrap()
    }

    if ccfg.qbe_file().is_none() {
        trace!("cleaning up {}", qbe_il_path.display());
        std::fs::remove_file(qbe_il_path).unwrap()
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

fn ccfg_out_dir(ccfg: &CompilerConfig) -> PathBuf {
    match ccfg.out_dir() {
        Ok(p) => p,
        Err(e) => {
            error!("cannot access current working directory: {}", e.kind());
            exit(1)
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

    let (scoper, status) = scope_module(module);
    let mut scoped_module = exit_on_user_error(status, &scoper);
    info!("commencing typechecking...");
    let mut tc = ZeaTypeChecker::new();
    let status = tc.typecheck_module(&mut scoped_module);
    exit_on_user_error(status, &tc);
    let tinfo = tc.finish();
    info!("finished typechecking");
    if ccfg.print_ipr() {
        info!("after expansions:\n{}", scoped_module.indent_print(0));
        info!("module Debug Print: {scoped_module:?}");
    }

    info!("lowering into THR...");
    let lowered = exit_on_ice(zea_irs::ast::thr::lower_module(
        scoped_module,
        tinfo,
        scoper,
        &ccfg,
    ));
    if ccfg.print_thr() {
        info!("Typed Highlevel Representation:\n{:?}", lowered);
    }

    let mut codegen = THRtoQBE::new(&lowered);
    let qbe = exit_on_ice(codegen.lower());
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
