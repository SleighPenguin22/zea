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
