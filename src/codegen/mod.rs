#![allow(unused)]
use crate::ast::zea::expression::Literal;
use crate::ast::zea::Type;
use crate::lowering::{ExpandedExpression, ExpandedStatement, SimpleInitialisation};

pub struct CondMatchFormatter {
    typ: Option<Type>,
    assignee: String,
    arms: Vec<ExpandedExpression>,
}

pub mod symbollowering;

pub fn canoncalize_zea_identifier(identifier: &str) -> String {
    identifier
        .replace("-", "_")
        .replace("!", "_bang_")
        .replace("?", "_maybe_")
        // .replace("__", "_")
        .trim_end_matches("_")
        .to_string()
}

pub trait CNode {
    fn emit_c(&self) -> String;
}

impl<T: CNode> CNode for Box<T> {
    fn emit_c(&self) -> String {
        self.as_ref().emit_c()
    }
}

impl CNode for ExpandedStatement {
    fn emit_c(&self) -> String {
        match self {
            // LoweredStatement::Initialisation(init) => init.emit_c(),
            ExpandedStatement::Return(expr) => format!("return ({});", expr.emit_c()),
            _ => unimplemented!("implement remaining lowered statement code generation"),
            // LoweredStatement::VoidReturn => "return;".to_string(),
            // LoweredStatement::FunctionCall(call) => call.emit_c() + ";",
            // LoweredStatement::Reassignment(reassignment) => reassignment.emit_c()
        }
    }
}

impl CNode for SimpleInitialisation {
    fn emit_c(&self) -> String {
        let typ = self
            .typ
            .as_ref()
            .expect("initialisation\n`{self:?}`\nshould have its type known before formatting.")
            .emit_c();

        format!("{typ} {} = {};", self.assignee, self.value.emit_c())
    }
}

impl CNode for ExpandedExpression {
    fn emit_c(&self) -> String {
        match self {
            ExpandedExpression::Literal(l) => l.emit_c(),
            _ => unimplemented!("remaining code generation for expressions"),
        }
    }
}

impl CNode for Literal {
    fn emit_c(&self) -> String {
        match self {
            Literal::Integer(i) => i.to_string() + "ull",
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            Literal::String(s) => "\"".to_string() + s + "\"",
        }
    }
}

impl CNode for Type {
    fn emit_c(&self) -> String {
        match self {
            Type::Basic(typ) => typ.clone(),
            _ => unimplemented!("implement remaining type formatting blabla"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::CNode;
    use crate::lowering::{ExpandedExpression, SimpleInitialisation};

    #[test]
    fn canonicalize_zea_identifier() {
        use super::canoncalize_zea_identifier as c;
        let s1 = "even?";
        let s2 = "kebab-case";
        let s3 = "map!";
        let s4 = "unify-types?!";
        let s5 = "unify-types?_!";
        assert_eq!(c(s1), "even_maybe");
        assert_eq!(c(s2), "kebab_case");
        assert_eq!(c(s3), "map_bang");
        assert_eq!(c(s4), "unify_types_maybe__bang");
        assert_eq!(c(s5), "unify_types_maybe___bang");
    }

    #[test]
    fn format_basic_init() {
        use crate::ast::zea::test_utils::*;
        let typ = types::int_type();
        let value: ExpandedExpression = literals::int_lit(3).into();

        let init = SimpleInitialisation::new(Some(typ), "a", value);

        assert_eq!(init.emit_c(), "I32 a = 3ull;")
    }
}
