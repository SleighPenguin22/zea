use crate::ast::patterns::ZeaPattern;
use crate::ast::statement::VarDecl;
use crate::ast::{VarDeclAssignment, ZeaExpression, ZeaTypeIdent};
use crate::codegen::CodegenResult;
use std::any::Any;
use std::fmt::Debug;

pub type LoweringResult<T> = Result<T, DesugaringError>;

mod error {
    use crate::ast::patterns::ZeaPattern;

    pub enum DesugaringError {
        InvalidPattern(ZeaPattern),
    }

    impl std::fmt::Display for DesugaringError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl std::fmt::Debug for DesugaringError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "{:?}",
                match self {
                    DesugaringError::InvalidPattern(p) => format!("InvalidPattern: {:?}", p),
                }
            )
        }
    }
}
use crate::codegen::expr::CExpr;
use error::DesugaringError;

pub trait DesugarInto {
    type LoweredOutput;

    fn desugar(self) -> LoweringResult<Self::LoweredOutput>;
}

pub trait CAssignment {
    fn c_assignee(&self, rhs: &ZeaExpression) -> CodegenResult<String>;
}

impl CAssignment for VarDeclAssignment {
    fn c_assignee(&self, rhs: &ZeaExpression) -> CodegenResult<String> {
        let lowered = self.desugar().map_err(|err| Err(err.to_string()))?;
        Ok(lowered
            .iter()
            .map(|assignment| assignment.c_stmt())
            .join(";\n"))
    }
}

pub struct LoweredVarDeclAssignment {
    pub lowered_var_decl: LoweredVarDecl,
    pub value: ZeaExpression,
}

impl LoweredVarDeclAssignment {
    const MUTABLE: &str = "";
    const IMMUTABLE: &str = "const ";
    pub fn format_mut_qualifier(&self) -> &str {
        if self.lowered_var_decl.mutable {
            Self::MUTABLE
        } else {
            Self::IMMUTABLE
        }
    }

    pub fn format_type_name(&self) -> &String {
        self.lowered_var_decl.typ.get_basic()
    }

    pub fn format_assignee(&self) -> String {
        self.lowered_var_decl
            .typ
            .format_assignee(self.lowered_var_decl.assignee.clone())
    }

    pub fn format_value(&self) -> CodegenResult<String> {
        self.value.c_expr()
    }
}

pub struct LoweredVarDecl {
    pub mutable: bool,
    pub typ: ZeaTypeIdent,
    pub assignee: String,
}

impl LoweredVarDecl {
    const MUTABLE: &str = "";
    const IMMUTABLE: &str = "const ";
    pub fn format_mut_qualifier(&self) -> &str {
        if self.mutable {
            Self::MUTABLE
        } else {
            Self::IMMUTABLE
        }
    }

    pub fn format_type_name(&self) -> &String {
        self.typ.get_basic()
    }

    pub fn format_assignee(&self) -> String {
        self.typ.format_assignee(self.assignee.clone())
    }
}

impl VarDeclAssignment {
    pub fn lower_basic_lhs(self) -> LoweringResult<LoweredVarDeclAssignment> {
        if let ZeaPattern::Ident(ref _ident) = self.decl.assignee {
            Ok(LoweredVarDeclAssignment {
                lowered_var_decl: self.decl.lower_basic()?,
                value: self.value,
            })
        } else {
            Err(DesugaringError::InvalidPattern(self.decl.assignee.clone()))
        }
    }
}

impl VarDecl {
    pub fn lower_basic(self) -> LoweringResult<LoweredVarDecl> {
        if let ZeaPattern::Ident(ident) = self.assignee {
            Ok(LoweredVarDecl {
                mutable: self.mutable,
                typ: self.typ,
                assignee: ident,
            })
        } else {
            Err(DesugaringError::InvalidPattern(self.assignee.clone()))
        }
    }
}

impl DesugarInto for VarDeclAssignment {
    type LoweredOutput = Vec<LoweredVarDeclAssignment>;
    fn desugar(self) -> LoweringResult<Self::LoweredOutput> {
        match self.decl.assignee {
            ZeaPattern::Ident(_) => Ok(vec![self.lower_basic_lhs()?]),
            _ => todo!("desugaring of destructured bindings in assigment"),
        }
    }
}
