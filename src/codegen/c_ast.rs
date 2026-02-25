#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    FuncCall(FunctionCall),
    Literal(Literal),
    Add(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    Sub(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    Mul(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    Div(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    Mod(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    Neg(Box<crate::ast::Expression>),

    LogAnd(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    LogOr(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    LogNot(Box<crate::ast::Expression>),

    BitAnd(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    BitOr(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    BitXor(Box<crate::ast::Expression>, Box<crate::ast::Expression>),
    BitNot(Box<crate::ast::Expression>),

    IfThenElse(Box<IfThenElse>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    pub condition: Expression,
    pub true_branch: Expression,
    pub false_branch: Expression,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(u64),
    Float(f64),
    Boolean(bool),
    String(String),
}
