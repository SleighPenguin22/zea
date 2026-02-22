mod error;

use crate::ast::patterns::ZeaPattern;
use crate::ast::statement::VarDecl;
use crate::ast::{VarDeclAssignment, ZeaExpression, ZeaStatement, ZeaTypeIdent};
use crate::codegen::CodegenResult;
use std::any::Any;
use std::fmt::Debug;

pub type LoweringResult<T> = Result<T, LoweringError>;

use crate::codegen::expr::CExpr;
use error::LoweringError;

/// Define some transformation on a node that desugars/lowers it into some other node
///
/// i.e. a destructuring assignment into multiple assignments.
pub trait LowerInto {
    type LoweredOutput;

    fn lower_into(self) -> LoweringResult<Self::LoweredOutput>;
}
#[derive(Debug, PartialEq, Clone)]
pub enum LoweredStatement {
    DeclAssign(LoweredVarDeclAssignment),
    Decl(LoweredVarDecl),
    ReturnValue(ZeaExpression),
    ReturnVoid,
}

impl From<LoweredVarDeclAssignment> for LoweredStatement {
    fn from(value: LoweredVarDeclAssignment) -> LoweredStatement {
        LoweredStatement::DeclAssign(value)
    }
}

impl From<LoweredVarDecl> for LoweredStatement {
    fn from(value: LoweredVarDecl) -> LoweredStatement {
        LoweredStatement::Decl(value)
    }
}

impl LowerInto for ZeaStatement {
    type LoweredOutput = Vec<LoweredStatement>;

    fn lower_into(self) -> LoweringResult<Self::LoweredOutput> {
        match self {
            ZeaStatement::VarDeclAssignment(vda) => Ok(vda
                .lower_into()?
                .into_iter()
                .map(LoweredStatement::from)
                .collect()),
            ZeaStatement::VarDecl(vd) => Ok(vd
                .lower_into()?
                .into_iter()
                .map(LoweredStatement::from)
                .collect()),
            _ => unimplemented!(
                "lowering is only implemented for declarations and assignments for now"
            ),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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

impl LowerInto for VarDecl {
    type LoweredOutput = Vec<LoweredVarDecl>;
    fn lower_into(self) -> LoweringResult<Vec<LoweredVarDecl>> {
        match self.assignee {
            ZeaPattern::Ident(_) => Ok(vec![self.lower_basic()?]),
            _ => unimplemented!("only basic non-destructuring assignments are allowed for now"),
        }
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
            Err(LoweringError::InvalidPattern(self.decl.assignee.clone()))
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
            Err(LoweringError::InvalidPattern(self.assignee.clone()))
        }
    }
}

impl LowerInto for VarDeclAssignment {
    type LoweredOutput = Vec<LoweredVarDeclAssignment>;
    fn lower_into(self) -> LoweringResult<Self::LoweredOutput> {
        match self.decl.assignee {
            ZeaPattern::Ident(_) => Ok(vec![self.lower_basic_lhs()?]),
            _ => todo!("desugaring of destructured bindings in assigment"),
        }
    }
}
