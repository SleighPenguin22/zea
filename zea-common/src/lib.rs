use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about)]
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
    #[arg(short, long = "save-asm")]
    asm_file: Option<PathBuf>,
    #[arg(short, long = "save-qbe")]
    qbe_file: Option<PathBuf>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerError {
    stage: CompilerStage,
    kind: CompilerErrorKind,
}
impl CompilerError {
    pub const fn new(stage: CompilerStage, kind: CompilerErrorKind) -> Self {
        Self { stage, kind }
    }
    pub fn pretty(&self) -> String {
        let s = self.stage;
        let k = self.kind;
        format!("INTERNAL COMPILER ERROR: {s:?} : {k:?}")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    Parse = 0,
    ExpandInit = 1,
    TypeCheck = 2,
    LexicalScopeAnalysis = 3,
    IPRtoTHR = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerErrorKind {
    IntegerOverflow = 0,

    StrayUnscopedIdent = 1,
    StrayPackedInit = 2,
}
pub fn stray_packed_init() -> CompilerError {
    CompilerError::new(
        CompilerStage::ExpandInit,
        CompilerErrorKind::StrayPackedInit,
    )
}
pub fn stray_unscoped_ident() -> CompilerError {
    CompilerError::new(
        CompilerStage::LexicalScopeAnalysis,
        CompilerErrorKind::StrayUnscopedIdent,
    )
}
#[macro_export]
macro_rules! internal_compiler_error {
    (spi) => {{
        use $crate::stray_packed_init;
        unreachable!("{}", stray_packed_init().pretty())
    }};
    (sui) => {{
        use $crate::stray_unscoped_ident;
        unreachable!("{}", stray_unscoped_ident().pretty())
    }};
}
