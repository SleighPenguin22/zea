use crate::ast::{Literal, ZeaExpression, ZeaStatement, ZeaTypeIdent};
pub mod statements {
    use super::*;
    use crate::ast::VarInitialisation;
    use crate::ast::patterns::ZeaPattern;
    use crate::ast::statement::VarDecl;

    /// A basic assignment like
    ///
    /// `const int a = 3;`
    /// `char b = 'b';`
    pub fn basic_assignment_mut(
        typ: ZeaTypeIdent,
        name: impl Into<String>,
        value: ZeaExpression,
    ) -> ZeaStatement {
        ZeaStatement::VarInitialisation(VarInitialisation {
            decl: VarDecl {
                mutable: true,
                typ,
                assignee: ZeaPattern::Ident(name.into()),
            },
            value,
        })
    }

    /// A basic assignment like
    ///
    /// `const int a = 3;`
    /// `char b = 'b';`
    pub fn basic_assignment_immut(
        typ: ZeaTypeIdent,
        name: impl Into<String>,
        value: ZeaExpression,
    ) -> ZeaStatement {
        ZeaStatement::VarInitialisation(VarInitialisation {
            decl: VarDecl {
                mutable: false,
                typ,
                assignee: ZeaPattern::Ident(name.into()),
            },
            value,
        })
    }
}

pub mod expressions {}

pub mod literals {
    use super::*;
    pub fn int_lit(value: u64) -> Literal {
        Literal::Integer(value)
    }

    pub fn float_lit(value: f64) -> Literal {
        Literal::Float(value)
    }

    pub fn bool_lit(value: bool) -> Literal {
        Literal::Boolean(value)
    }
    pub fn str_lit(value: impl Into<String>) -> Literal {
        Literal::String(value.into())
    }
}
pub mod types {
    use crate::ast::ZeaTypeIdent;

    pub fn ptr_to(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Ptr(Box::new(typ))
    }
    pub fn array_of(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::ArrayOf(Box::new(typ))
    }

    pub fn slice_of(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Slice(Box::new(typ))
    }

    pub fn opt_type(typ: ZeaTypeIdent) -> ZeaTypeIdent {
        ZeaTypeIdent::Option(Box::new(typ))
    }

    pub fn str_type() -> ZeaTypeIdent {
        ZeaTypeIdent::Basic("Str".into())
    }

    pub fn int_type() -> ZeaTypeIdent {
        ZeaTypeIdent::Basic("I32".into())
    }

    pub fn uint_type() -> ZeaTypeIdent {
        ZeaTypeIdent::Basic("U32".into())
    }

    pub fn float_type() -> ZeaTypeIdent {
        ZeaTypeIdent::Basic("F32".into())
    }

    pub fn bool_type() -> ZeaTypeIdent {
        ZeaTypeIdent::Basic("Bool".into())
    }
}
