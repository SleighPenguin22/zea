#![allow(dead_code)]
use crate::ast::statement::ZeaStatement;
use crate::eval::ZeaModuleEvaluator;
use crate::eval::evalexpr::ZeaEvalError;
use std::io::{Read, Write};

pub trait EvalStmt {
    fn eval_stmt<I: std::io::Read, O: std::io::Write>(
        &self,
        context: &mut ZeaModuleEvaluator<I, O>,
    ) -> Result<(), ZeaEvalError>;
}
impl EvalStmt for ZeaStatement {
    fn eval_stmt<I: Read, O: Write>(
        &self,
        _context: &mut ZeaModuleEvaluator<I, O>,
    ) -> Result<(), ZeaEvalError> {
        match self {
            _ => todo!(),
        }
    }
}
