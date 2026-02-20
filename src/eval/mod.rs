#![allow(dead_code)]
mod evalexpr;
mod evalstmt;
mod typecheck;
mod virtualmachine;

use crate::ast::datatype::ZeaTypeIdent;
use crate::ast::expression::ZeaExpression;
use crate::ast::{FuncDeclaration, TopLevelStatement, ZeaModule};
pub use crate::eval::evalexpr::{EvaluationScheme, ZeaEvalError};
use crate::eval::typecheck::ZeaTypeObject;
use std::collections::HashMap;
use std::io;
use std::io::{BufReader, BufWriter};

pub struct ZeaModuleEvaluator<IOIn: io::Read, IOOut: io::Write> {
    module: ZeaModule,
    const_fold_cache: HashMap<ZeaExpression, ZeaExpression>,
    stdout: BufWriter<IOOut>,
    stdin: BufReader<IOIn>,
}

pub enum ZeaModuleEvaluatorError {
    IoError(io::Error),
    EvalError(ZeaEvalError),
    EntryPointNotFound,
}

/// Some value annotated with a type
pub struct ZeaValue<T: Sized> {
    typ: ZeaTypeObject,
    value: T,
}
impl<T> ZeaValue<T> {
    pub fn var(typ: ZeaTypeIdent, value: T) -> Self {
        Self {
            typ: ZeaTypeObject::VarType(typ),
            value,
        }
    }

    pub fn func(typ: FuncDeclaration, value: T) -> Self {
        Self {
            typ: ZeaTypeObject::FuncSig(typ),
            value,
        }
    }

    pub fn get_type(&self) -> &ZeaTypeObject {
        &self.typ
    }
}

pub trait MessageWrapper {
    fn msg_wrapper(&self, message: impl Into<String>) -> impl Fn(String) -> String;
}

impl<I: io::Read, O: io::Write> ZeaModuleEvaluator<I, O> {
    fn new(input_stream: I, output_stream: O) -> Self {
        ZeaModuleEvaluator {
            const_fold_cache: HashMap::new(),
            stdin: BufReader::new(input_stream),
            stdout: BufWriter::new(output_stream),
            module: Default::default(),
        }
    }
}

impl ZeaModule {
    pub fn iter_symbols(&self) -> impl Iterator<Item = &TopLevelStatement> {
        self.symbols.iter()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn boolplusbool() {}
}
