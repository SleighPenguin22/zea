use crate::datatype::ZeaType;
use crate::expression::{Literal, ZeaExpression};
use crate::patterns::ZeaPattern;
use crate::statement::{VarDecl, ZeaStatement};
use std::collections::HashSet;



impl ZeaPattern {
    pub fn ident(s: impl Into<String>) -> ZeaPattern {
        ZeaPattern::Ident(s.into())
    }
}

impl ZeaStatement {
    pub fn const_decl(s: impl Into<String>, typ: impl Into<ZeaType>) -> ZeaStatement {
        VarDecl {
            assignee: ZeaPattern::ident(s),
            mutable: false,
            storage_qualifiers: HashSet::new(),
        }
        .into()
    }

    pub fn var_decl(s: impl Into<String>, typ: impl Into<ZeaType>) -> ZeaStatement {
        VarDecl {
            assignee: ZeaPattern::ident(s),
            mutable: true,
            storage_qualifiers: HashSet::new(),
        }
        .into()
    }
}

