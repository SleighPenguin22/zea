use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about, name = "zea")]
pub struct CompilerConfig {
    /// The source file to compile
    infile_path: PathBuf,

    #[arg(short,long, default_value_t = log::LevelFilter::Info, rename_all="lower")]
    /// The tracing-verbosity
    verbosity: LevelFilter,

    #[arg(long, default_value_t = false)]
    /// Dump the generated THR before lowering into QBE-IL
    dump_thr: bool,

    #[arg(long, default_value_t = false)]
    /// Dump the parsed Module before lowering into THR
    dump_ipr: bool,

    #[arg(long, default_value_t = false)]
    /// Dump the generated QBE-IL
    print_qbe_il: bool,

    #[arg(short = 'o', long = "outfile", default_value = None)]
    /// the filename to write the final object to
    outfile_path: Option<PathBuf>,

    #[arg(long = "artifact-dir")]
    /// Specify a directory-path to place artifacts in, defaults to the working directory
    out_dir: Option<PathBuf>,

    #[arg(long = "asm")]
    /// Save the generated assembly, optionally to a named file
    asmfile_path: Option<Option<PathBuf>>,

    #[arg(long = "qbe")]
    /// Save the generated QBE-IL, optionally to a named file
    qbefile_path: Option<Option<PathBuf>>,

    #[arg(long = "object-type", default_value = "bin", rename_all = "lower")]
    /// Specify the object type this module should compile to
    object_type: ModuleType,
    #[arg(long, default_value_t = false)]
    /// If set, keep assembly and QBE artifacts
    keep_files: bool,
    #[arg(long = "emit", default_value = "asm", rename_all = "lower")]
    /// Specify the format this module should compile to,
    /// the compiler stops the pipeline after reaching this stage
    emit: ModuleEmissionType,
}

impl CompilerConfig {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn get_log_level(&self) -> LevelFilter {
        let cmp = if self.dump_thr || self.dump_ipr {
            LevelFilter::Info
        } else {
            LevelFilter::Off
        };
        std::cmp::max(cmp, self.verbosity)
    }

    pub fn should_print_thr(&self) -> bool {
        self.dump_thr
    }

    pub fn get_infile_path(&self) -> &PathBuf {
        &self.infile_path
    }

    /// Get the filename of the to-emit assembly file
    pub fn get_asmfile_name(&self) -> PathBuf {
        match self.asmfile_path.as_ref() {
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
            Some(p) => p.clone(),
            _ => {
                let mut infile = self.infile_path.clone();
                infile.set_extension("out");
                infile
            }
        }
    }

    /// Get the filename of the to-emit QBE-IL file
    pub fn get_qbefile_name(&self) -> PathBuf {
        match self.qbefile_path.as_ref() {
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
        self.dump_ipr
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

    pub fn should_invoke_qbe(&self) -> bool {
        self.emit >= ModuleEmissionType::QBE
    }
    pub fn should_invoke_assembler(&self) -> bool {
        self.emit >= ModuleEmissionType::ASM
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum, Default)]
pub enum ModuleType {
    /// The target object should be an executable
    #[default]
    Bin,
    /// The target object should be a static library
    Lib,
    /// The target object should be a dynamic library
    Dll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default, PartialOrd, Ord)]
pub enum ModuleEmissionType {
    /// The compilation unit should be compiled down to QBE IL
    QBE = 1,
    /// The compilation unit should be compiled down to Assembly
    #[default]
    ASM = 2,
    /// The compilation unit should be compiled all the way to an ELF object
    ObjectFile = 3,
}
