use crate::statement::FuncCall;

pub enum ZeaExpression {
    FuncCall(FuncCall),
    Literal(Literal),
}

pub enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl Into<Literal> for i64 {
    fn into(self) -> Literal {
        Literal::Integer(self)
    }
}
impl Into<Literal> for f64 {
    fn into(self) -> Literal {
        Literal::Float(self)
    }
}
impl Into<Literal> for bool {
    fn into(self) -> Literal {
        Literal::Boolean(self)
    }
}

impl Into<Literal> for String {
    fn into(self) -> Literal {
        Literal::String(self)
    }
}
