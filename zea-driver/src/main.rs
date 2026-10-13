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
use zea_common::{CompilerConfig, InternalCompilerError, ZeaError, ZeaUserError};
use zea_irs::ast::{
    NodeLabeler,
    ipr_walkers::{
        self, IPRTransfomer,
        transformers::{self, AssignmentExpander, IdentifierScoper, scope_module},
    },
    thr::{THRKeySets, THRModule},
};
use zea_irs::visualisation::IndentPrint;
use zea_irs::{ZeaTypeChecker, typecheck_module};
use zea_parser::parse_module;

/// 'unwrap' an error, providing context that the user-error needs
/// as to supply the user with helpful info,
///
/// In the case of an internal compiler error (ICE),
/// bail with a message and cause.
fn error_unpack_ctx<T, UserErr: ZeaUserError>(
    res: Result<T, ZeaError<UserErr>>,
    ctx: &UserErr::ZeaUserErrorContext,
) -> T {
    match res {
        Ok(t) => t,
        Err(ZeaError::User(e)) => {
            let msg = e.zea_error_format(ctx);
            error!("{msg}");
            exit(1);
        }
        Err(ZeaError::Internal(ice)) => {
            let msg = ice.pretty();
            error!("{msg}");
            exit(2);
        }
    }
}

/// A shorthand for error_unpack for error types whose context type is `()`
fn error_unpack<T, UserErr>(res: Result<T, ZeaError<UserErr>>) -> T
where
    UserErr: ZeaUserError<ZeaUserErrorContext = ()>,
{
    error_unpack_ctx(res, &())
}
/// 'unwrap' an internal compiler error (ICE),
/// bail with a message and cause.
fn error_unpack_ice<T>(res: Result<T, InternalCompilerError>) -> T {
    match res {
        Ok(t) => t,
        Err(ice) => {
            let msg = ice.pretty();
            error!("{msg}");
            exit(2);
        }
    }
}

fn create_file(filename: &Path) -> (File, PathBuf) {
    let path = PathBuf::from(filename);
    let temp_file = File::create(&path).unwrap();
    (temp_file, path)
}

/// Invoke GCC's assembler and return the path that the artifact got written to.
fn invoke_assembler(
    ccfg: &CompilerConfig,
    asm_path: &Path,
) -> Result<PathBuf, ZeaError<std::io::Error>> {
    let out_path = ccfg.get_prefixed_outfile_path()?;
    let status = std::process::Command::new("gcc")
        .arg(asm_path)
        .arg("-o")
        .arg(&out_path)
        .arg("-Wall")
        .status()?;

    if !status.success()
        && let Some(code) = status.code()
    {
        error!("GCC returned error code {code}, exiting...");
        exit(code)
    }
    info!("saved compiled binary to {}", out_path.display());
    Ok(out_path)
}
/// Invoke QBE and return the path that the artifact got written to.
fn invoke_qbe(ccfg: &CompilerConfig, qbe_path: &Path) -> Result<PathBuf, ZeaError<std::io::Error>> {
    let asmpath = ccfg.get_prefixed_asmfile_path()?;
    let status = std::process::Command::new("qbe")
        .arg(qbe_path)
        .arg("-o")
        .arg(&asmpath)
        .status()?;

    if !status.success()
        && let Some(code) = status.code()
    {
        error!("QBE returned error code {code}, exiting...");
        exit(code)
    }
    info!("saved QBE assembly to {}", asmpath.display());
    Ok(asmpath)
}
fn write_qbe_il(ccfg: &CompilerConfig, il_str: &str) -> Result<PathBuf, ZeaError<std::io::Error>> {
    let qbefile_path = ccfg.get_prefixed_qbefile_path()?;
    let (mut file, path) = create_file(qbefile_path.as_path());
    file.write_all(il_str.as_bytes()).unwrap();
    Ok(path)
}

/// remove any artifact that the config specified not to keep.
fn cleanup_temp_files(
    ccfg: &CompilerConfig,
    qbe_il_path: &Path,
    asm_path: &Path,
) -> Result<(), ZeaError<std::io::Error>> {
    if ccfg.should_keep_files() {
        return Ok(());
    }
    if !ccfg.defines_asmfile() {
        trace!("cleaning up {}", asm_path.display());
        std::fs::remove_file(asm_path)?;
    }

    if !ccfg.defines_qbefile() {
        trace!("cleaning up {}", qbe_il_path.display());
        std::fs::remove_file(qbe_il_path)?;
    }
    Ok(())
}
fn exit(code: i32) -> ! {
    error!("exiting...");
    pexit(code)
}

fn read_to_string_wrapper(p: &Path) -> Result<String, ZeaError<std::io::Error>> {
    let s = read_to_string(p)?;
    Ok(s)
}

fn main() {
    let ccfg = CompilerConfig::parse_args();
    colog::basic_builder()
        .filter_level(ccfg.get_log_level())
        .init();
    let src = read_to_string_wrapper(ccfg.get_infile_path());
    let src = error_unpack_ctx(src, &());
    let (mut module, labeler) = parse_module(&src);

    let mut expander = transformers::AssignmentExpander::from_labeler(labeler);
    expander.visit_module(&mut module).expect("infallible");

    let mut insert_main_return0 = transformers::InsertImplicitMainReturn::from_labeler(expander);
    insert_main_return0
        .visit_module(&mut module)
        .expect("infallible");

    let mut scoper = IdentifierScoper::new(&module);
    let status = scoper.scope_module(module);
    let mut scoped_module = error_unpack_ctx(status, &scoper);

    info!("commencing typechecking...");
    let mut tc = ZeaTypeChecker::new();
    let status = tc.typecheck_module(&mut scoped_module);
    error_unpack_ctx(status, &tc);
    let tinfo = tc.finish();
    info!("finished typechecking");
    if ccfg.should_print_ipr() {
        info!("after expansions:\n{}", scoped_module.indent_print(0));
    }

    info!("lowering into THR...");
    let thr_ctx = THRKeySets::new();
    let status = thr_ctx.lower_module(scoped_module, tinfo, scoper, &ccfg);
    let lowered = error_unpack_ice(status);
    if ccfg.should_print_thr() {
        info!("Typed Highlevel Representation:\n{:?}", lowered);
    }

    let mut codegen = THRtoQBE::new(&lowered);
    let qbe = error_unpack_ice(codegen.lower());
    let il = format!("{qbe}");
    if ccfg.should_print_qbe_il() {
        info!("Generated QBE IL:\n{il}");
    }

    let qbe_path = error_unpack(write_qbe_il(&ccfg, &il));

    let asm_path = error_unpack(invoke_qbe(&ccfg, &qbe_path));
    let _out_path = error_unpack(invoke_assembler(&ccfg, &asm_path));

    info!(
        "saved compiled binary to {}",
        _out_path.canonicalize().unwrap().display()
    );
    error_unpack(cleanup_temp_files(&ccfg, &qbe_path, &asm_path));
}
