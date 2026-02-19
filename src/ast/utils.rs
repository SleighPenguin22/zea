use crate::ast::datatype::ZeaTypeIdent;
use crate::ast::patterns::ZeaPattern;
use crate::ast::statement::{VarDecl, ZeaStatement};
use std::collections::HashSet;

impl ZeaPattern {
    pub fn ident(s: impl Into<String>) -> ZeaPattern {
        ZeaPattern::Ident(s.into())
    }
}

impl ZeaStatement {
    pub fn const_decl(s: impl Into<String>, typ: impl Into<ZeaTypeIdent>) -> ZeaStatement {
        VarDecl {
            assignee: ZeaPattern::ident(s),
            mutable: false,
        }
        .into()
    }

    pub fn var_decl(s: impl Into<String>, typ: impl Into<ZeaTypeIdent>) -> ZeaStatement {
        VarDecl {
            assignee: ZeaPattern::ident(s),
            mutable: true,
        }
        .into()
    }
}
