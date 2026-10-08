use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about, name = "zea")]
pub struct CompilerConfig {
    path: PathBuf,

    #[arg(long = "loglevel", default_value_t = log::LevelFilter::Info)]
    log_level: LevelFilter,

    #[arg(long = "print-thr", default_value_t = false)]
    print_thr: bool,
    #[arg(long = "print-ipr", default_value_t = false)]
    print_ipr: bool,
    #[arg(long = "print-qbe-il", default_value_t = false)]
    print_qbe_il: bool,

    #[arg(short, long = "output")]
    out_file: Option<PathBuf>,
    #[arg(long = "out-dir")]
    out_dir: Option<PathBuf>,
    #[arg(short, long = "save-asm")]
    asm_file: Option<PathBuf>,
    #[arg(short, long = "save-qbe")]
    qbe_file: Option<PathBuf>,
    #[arg(long, default_value_t = ModuleType::Bin)]
    target_type: ModuleType,
    #[arg(long, default_value_t = false)]
    keep_files: bool,
}

impl CompilerConfig {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn log_level(&self) -> LevelFilter {
        let cmp = if self.print_thr || self.print_ipr {
            LevelFilter::Info
        } else {
            LevelFilter::Off
        };
        std::cmp::max(cmp, self.log_level)
    }

    pub fn print_thr(&self) -> bool {
        self.print_thr
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn asm_file(&self) -> Option<&PathBuf> {
        self.asm_file.as_ref()
    }

    pub fn out_file(&self) -> Option<&PathBuf> {
        self.out_file.as_ref()
    }

    pub fn qbe_file(&self) -> Option<&PathBuf> {
        self.qbe_file.as_ref()
    }

    pub fn print_ipr(&self) -> bool {
        self.print_ipr
    }

    pub fn print_qbe_il(&self) -> bool {
        self.print_qbe_il
    }

    pub fn module_type(&self) -> ModuleType {
        self.target_type
    }
    pub fn out_dir(&self) -> std::io::Result<PathBuf> {
        match &self.out_dir {
            Some(path) => Ok(path.clone()),
            None => std::env::current_dir(),
        }
    }

    pub fn keep_files(&self) -> bool {
        self.keep_files
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum, Default)]
pub enum ModuleType {
    #[default]
    Bin,
    Lib,
    Dll,
}

impl ToString for ModuleType {
    fn to_string(&self) -> String {
        match self {
            ModuleType::Bin => "bin",
            ModuleType::Lib => "lib",
            ModuleType::Dll => "dll",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct CompilerError {
    pub stage: CompilerStage,
    pub kind: CompilerErrorKind,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZeaErrorMessage {
    pub msg: String,
}
impl std::fmt::Display for ZeaErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

pub trait ZeaError<T>: Sized {
    fn zea_error_format(&self, ctx: &T) -> ZeaErrorMessage;
    fn into_zea_error(self, ctx: &T, stage: CompilerStage) -> CompilerError {
        let msg = self.zea_error_format(ctx);
        CompilerError {
            stage,
            kind: CompilerErrorKind::Other(msg),
        }
    }
}
impl CompilerError {
    pub const fn new(stage: CompilerStage, kind: CompilerErrorKind) -> Self {
        Self { stage, kind }
    }
    pub fn pretty(&self) -> String {
        let s = self.stage;
        let k = &self.kind;
        format!("INTERNAL COMPILER ERROR: {s:?} : {k:?}")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    NonApplicable = 0,
    Parse = 1,
    ExpandInit = 2,
    TypeCheck = 3,
    LexicalScopeAnalysis = 4,
    IPRtoTHR = 5,
    CodeGen = 6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerErrorKind {
    IntegerOverflow,

    StrayUnscopedIdent,
    StrayPackedInit,
    GlobalNonInitStmt,
    InvalidNonGlobalSymbolUsage,
    /// A typ field is set to None after the type-checking phase
    StrayUnknownType,
    EmptyFunctionBody,
    NoEntryPoint,
    Other(ZeaErrorMessage),
}
#[macro_export]
macro_rules! ice_bail {
    ($err:expr) => {{
        let err: CompilerError = $err;
        log::error!("{}", err.pretty());
        exit(1)
    }};
}
