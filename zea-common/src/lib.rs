use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about, name = "zea")]
pub struct CompilerConfig {
    infile_path: PathBuf,

    #[arg(long = "loglevel", default_value_t = log::LevelFilter::Info)]
    log_level: LevelFilter,

    #[arg(long = "print-thr", default_value_t = false)]
    print_thr: bool,
    #[arg(long = "print-ipr", default_value_t = false)]
    print_ipr: bool,
    #[arg(long = "print-qbe-il", default_value_t = false)]
    print_qbe_il: bool,

    #[arg(short = 'o', long = "output", default_value = None)]
    outfile_path: Option<Option<PathBuf>>,
    #[arg(long = "out-dir")]
    out_dir: Option<PathBuf>,
    #[arg(long = "save-asm")]
    asmfile_path: Option<Option<PathBuf>>,
    #[arg(long = "save-qbe")]
    qbefile_path: Option<Option<PathBuf>>,
    #[arg(long = "object-type", default_value_t = ModuleType::Bin)]
    object_type: ModuleType,
    #[arg(long, default_value_t = false)]
    keep_files: bool,
}

impl CompilerConfig {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn get_log_level(&self) -> LevelFilter {
        let cmp = if self.print_thr || self.print_ipr {
            LevelFilter::Info
        } else {
            LevelFilter::Off
        };
        std::cmp::max(cmp, self.log_level)
    }

    pub fn should_print_thr(&self) -> bool {
        self.print_thr
    }

    pub fn get_infile_path(&self) -> &PathBuf {
        &self.infile_path
    }

    /// Get the filename of the to-emit assembly file
    pub fn get_asmfile_name(&self) -> PathBuf {
        match self.outfile_path.as_ref() {
            Some(Some(p)) => p.clone(),
            _ => {
                let mut infile = self.infile_path.clone();
                infile.set_extension("S");
                infile
            }
        }
    }

    /// Get the filename of the to-emit final object
    pub fn get_outfile_name(&self) -> PathBuf {
        match self.outfile_path.as_ref() {
            Some(Some(p)) => p.clone(),
            _ => {
                let mut infile = self.infile_path.clone();
                infile.set_extension("out");
                infile
            }
        }
    }

    /// Get the filename of the to-emit QBE-IL file
    pub fn get_qbefile_name(&self) -> PathBuf {
        match self.outfile_path.as_ref() {
            Some(Some(p)) => p.clone(),
            _ => {
                let mut infile = self.infile_path.clone();
                infile.set_extension("qbe");
                infile
            }
        }
    }
    /// Get the filename of the to-emit QBE-IL file, prefixed with the artifact directory
    pub fn get_prefixed_qbefile_path(&self) -> std::io::Result<PathBuf> {
        let outdir = self.out_dir()?;
        Ok(outdir.join(self.get_qbefile_name()))
    }
    /// Returns true if the ccfg defined a QBE-IL file name
    pub fn defines_qbefile(&self) -> bool {
        self.qbefile_path.is_some()
    }
    /// Get the filename of the to-emit assembly file, prefixed with the artifact directory
    pub fn get_prefixed_asmfile_path(&self) -> std::io::Result<PathBuf> {
        let outdir = self.out_dir()?;
        Ok(outdir.join(self.get_asmfile_name()))
    }
    /// Returns true if the ccfg defined an assembly file name
    pub fn defines_asmfile(&self) -> bool {
        self.asmfile_path.is_some()
    }
    /// Get the filename of the to-emit ouput object, prefixed with the artifact directory
    pub fn get_prefixed_outfile_path(&self) -> std::io::Result<PathBuf> {
        let outdir = self.out_dir()?;
        Ok(outdir.join(self.get_outfile_name()))
    }

    /// Returns true if the expanded IPR should be dumped
    pub fn should_print_ipr(&self) -> bool {
        self.print_ipr
    }

    /// Returns true if generated QBE-IL should be dumped
    pub fn should_print_qbe_il(&self) -> bool {
        self.print_qbe_il
    }

    /// get the type of object that this compilation unit should compile to
    pub fn get_module_type(&self) -> ModuleType {
        self.object_type
    }
    /// Get the artifact directory, or the CWD if none is specified
    pub fn out_dir(&self) -> std::io::Result<PathBuf> {
        match &self.out_dir {
            Some(path) => Ok(path.clone()),
            None => std::env::current_dir(),
        }
    }
    /// Returns true if the generated assembly and QBE files should not be deleted.
    pub fn should_keep_files(&self) -> bool {
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

#[derive(Debug, Clone)]
pub enum ZeaError<E>
where
    E: ZeaUserError,
{
    User(E),
    Internal(InternalCompilerError),
}

impl<E> From<InternalCompilerError> for ZeaError<E>
where
    E: ZeaUserError,
{
    fn from(v: InternalCompilerError) -> Self {
        Self::Internal(v)
    }
}

impl<UserErr> From<UserErr> for ZeaError<UserErr>
where
    UserErr: ZeaUserError,
{
    fn from(v: UserErr) -> Self {
        Self::User(v)
    }
}

/// A Zea User error that requires some context to print a message
///
///
pub trait ZeaUserError: Sized {
    type ZeaUserErrorContext: ?Sized;
    fn zea_error_format(&self, ctx: &Self::ZeaUserErrorContext) -> ZeaErrorMessage;
    fn into_zea_error(self) -> ZeaError<Self> {
        ZeaError::User(self)
    }
}

impl ZeaUserError for std::io::Error {
    type ZeaUserErrorContext = ();

    fn zea_error_format(&self, _ctx: &Self::ZeaUserErrorContext) -> ZeaErrorMessage {
        let kind = self.kind();
        ZeaErrorMessage {
            msg: kind.to_string(),
            loc: None,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct InternalCompilerError {
    pub stage: CompilerStage,
    pub kind: CompilerErrorKind,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZeaErrorMessage {
    pub msg: String,
    pub loc: Option<(usize, usize)>,
}
impl std::fmt::Display for ZeaErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}
impl InternalCompilerError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

type ZeaResult<T, E> = Result<T, ZeaError<E>>;

#[macro_export]
macro_rules! ice_bail {
    ($err:expr) => {{
        let err: InternalCompilerError = $err;
        log::error!("{}", err.pretty());
        exit(1)
    }};
}
