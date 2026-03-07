pub mod statements {}

pub mod expressions {}

pub mod literals {
    use crate::ast::zea::expression::Literal;
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
    use crate::ast::zea::Type;

    pub fn ptr_to(typ: Type) -> Type {
        Type::Pointer(Box::new(typ))
    }
    pub fn array_of(typ: Type) -> Type {
        Type::ArrayOf(Box::new(typ))
    }

    pub fn str_type() -> Type {
        Type::Basic("Str".into())
    }

    pub fn int_type() -> Type {
        Type::Basic("I32".into())
    }

    pub fn uint_type() -> Type {
        Type::Basic("U32".into())
    }

    pub fn float_type() -> Type {
        Type::Basic("F32".into())
    }

    pub fn bool_type() -> Type {
        Type::Basic("Bool".into())
    }
}
