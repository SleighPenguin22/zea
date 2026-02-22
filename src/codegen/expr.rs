use crate::ast;
use crate::ast::ZeaExpression;
use crate::ast::statement::FuncCall;
use crate::codegen::CodegenResult;
use crate::codegen::error::CodeGenError;
use std::fmt::format;

pub trait CExpr {
    fn c_expr(&self) -> crate::codegen::CodegenResult<String>;

    fn insert_in_template(
        &self,
        template: impl Fn(String) -> String,
    ) -> crate::codegen::CodegenResult<String> {
        let s = self.c_expr()?;
        Ok(template(s))
    }
}

impl CExpr for ast::Literal {
    fn c_expr(&self) -> CodegenResult<String> {
        use ast::Literal;
        match self {
            Literal::Integer(int) => Ok(int.to_string()),
            Literal::Float(float) => Ok(float.to_string()),
            Literal::String(string) => Ok(format!("\"{}\"", string)),
            Literal::Boolean(bool) => Ok(if *bool { 1.to_string() } else { 0.to_string() }),
        }
    }
}

impl CExpr for ZeaExpression {
    fn c_expr(&self) -> CodegenResult<String> {
        match self {
            ZeaExpression::Literal(lit) => lit.c_expr(),
            ZeaExpression::Add(lhs, rhs) => Ok(format!("({} + {})", lhs.c_expr()?, rhs.c_expr()?)),
            ZeaExpression::Sub(lhs, rhs) => Ok(format!("({} - {})", lhs.c_expr()?, rhs.c_expr()?)),
            ZeaExpression::Mul(lhs, rhs) => Ok(format!("({} * {})", lhs.c_expr()?, rhs.c_expr()?)),
            ZeaExpression::Div(lhs, rhs) => Ok(format!("({} / {})", lhs.c_expr()?, rhs.c_expr()?)),
            ZeaExpression::IfThenElse(cond, then, els) => {
                Ok(format!("(({}) ? ({}) : ({}))", cond.c_expr()?, then, els))
            }
            ZeaExpression::FuncCall(FuncCall { name, args }) => {
                let args = args
                    .iter()
                    .map(|e| e.c_expr())
                    .collect::<Result<Vec<_>, _>>()?;
                let args = args.join(", ");

                Ok(format!("{}({})", name, args))
            }
            _ => todo!("implement remaining codegen, namely for {:?}", self),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::ast::utils::{expressions as ZE, literals as ZL};
    use crate::codegen::expr::CExpr;

    #[test]
    fn literal_expression() {
        let b1 = ZL::literal_bool(true);
        let b2 = ZL::literal_bool(false);

        let i0 = ZL::literal_int(0);
        let i1 = ZL::literal_int(1);
        let imin = ZL::literal_int(u64::MIN);
        let imax = ZL::literal_int(u64::MAX);

        assert_eq!(b1.c_expr().unwrap(), "1");
        assert_eq!(b2.c_expr().unwrap(), "0");

        assert_eq!(i0.c_expr().unwrap(), "0");
        assert_eq!(i1.c_expr().unwrap(), "1");
        assert_eq!(imin.c_expr().unwrap(), "0");
        assert_eq!(imax.c_expr().unwrap(), "18446744073709551615");
    }
}
