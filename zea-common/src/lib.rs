use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct CompilerConfig {
    path: PathBuf,

    #[arg(long = "loglevel", default_value_t = log::LevelFilter::Error)]
    log_level: LevelFilter,

    #[arg(long = "print-mir", default_value_t = false)]
    print_mir: bool,
}

impl CompilerConfig {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }

    pub fn print_mir(&self) -> bool {
        self.print_mir
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
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
    IPRtoTHR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerErrorKind {
    IntegerOverflow = 0,

    StrayUnscopedIdent = 1,
    StrayPackedInit = 2,
}
