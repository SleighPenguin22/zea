#![allow(unused)]

use zea_ast::c::datatype::TypeSpecifier;
use zea_ast::c::expression::Literal;
use zea_ast::c::statement::{Initialisation, Statement, StatementBlock, VariableDeclaration};
use zea_ast::c::{
    Expression, FunctionDeclaration, FunctionDefinition, Reassignment, TypedIdentifier,
};

pub mod node_c_conversion;

pub fn canoncalize_zea_identifier(identifier: &str) -> String {
    identifier
        .replace("-", "_")
        .replace("!", "_bang_")
        .replace("?", "_maybe_")
        // .replace("__", "_")
        .trim_end_matches("_")
        .to_string()
}

pub trait EmitC {
    fn emit_c(&self) -> String;
}

pub trait ConvertC {
    fn convert_c(self) -> Vec<Box<dyn EmitC>>;
}

impl<T: EmitC> EmitC for Box<T> {
    fn emit_c(&self) -> String {
        self.as_ref().emit_c()
    }
}

impl EmitC for Statement {
    fn emit_c(&self) -> String {
        match self {
            // LoweredStatement::Initialisation(init) => init.emit_c(),
            Statement::VariableDeclaration(vardecl) => vardecl.emit_c(),
            Statement::VariableInitialisation(init) => init.emit_c(),
            Statement::Return(expr) => format!("return {};", expr.emit_c()),
            _ => unimplemented!("cannot yet generate code for statement:\n{self:?}\n"),
            // LoweredStatement::VoidReturn => "return;".to_string(),
            // LoweredStatement::FunctionCall(call) => call.emit_c() + ";",
            // LoweredStatement::Reassignment(reassignment) => reassignment.emit_c()
        }
    }
}

pub fn fold_str(iter: impl Iterator<Item = String>, with: &str) -> String {
    iter.collect::<Vec<_>>().join(with)
}

pub fn fmt_typed_assignee(typename: &TypeSpecifier, assignee: &str) -> String {
    fn fmt_assignee(typename: &TypeSpecifier, assignee: &str) -> String {
        match typename {
            TypeSpecifier::Basic(t) => canoncalize_zea_identifier(assignee),
            TypeSpecifier::Pointer(t) => fmt_assignee(t.as_ref(), &format!("*{assignee}")),
        }
    }

    let assignee = fmt_assignee(typename, assignee);
    format!("{} {assignee}", typename.get_deepest())
}

impl EmitC for VariableDeclaration {
    fn emit_c(&self) -> String {
        let qualifiers = fold_str(self.typ.qualifiers.iter().map(String::from), " ");
        let qualifiers = if qualifiers.is_empty() {
            String::from("")
        } else {
            format!("{qualifiers} ")
        };

        let typed_ident = fmt_typed_assignee(&self.typ.specifier, &self.name);

        format!("{qualifiers}{typed_ident};")
    }
}

impl EmitC for Initialisation {
    fn emit_c(&self) -> String {
        let typed_ident = TypedIdentifier(self.typ.clone(), self.name.clone()).emit_c();

        let value = self.value.emit_c();

        format!("{typed_ident} = {value};")
    }
}

impl EmitC for Literal {
    fn emit_c(&self) -> String {
        match self {
            Literal::Integer(i) => format!("({i}ull)"),
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            Literal::String(s) => format!("\"{s}\""),
        }
    }
}

impl EmitC for Expression {
    fn emit_c(&self) -> String {
        match self {
            Expression::Literal(l) => l.emit_c(),
            Expression::Ident(i) => i.to_owned(),
            _ => unimplemented!("cannot yet generate code for expression:\n{self:?}\n\n"),
        }
    }
}

impl EmitC for TypedIdentifier {
    fn emit_c(&self) -> String {
        let mut qualifiers = fold_str(self.0.qualifiers.iter().map(String::from), " ");
        let qualifiers = if qualifiers.is_empty() {
            ""
        } else {
            &format!("{qualifiers} ")
        };

        let type_ident = fmt_typed_assignee(&self.0.specifier, &self.1);

        format!("{qualifiers}{type_ident}")
    }
}

impl EmitC for FunctionDeclaration {
    fn emit_c(&self) -> String {
        let type_name = TypedIdentifier(self.returns.clone(), self.name.clone()).emit_c();
        let args = fold_str(self.args.iter().map(|arg| arg.emit_c()), ", ");

        format!("{type_name}({args});")
    }
}

impl EmitC for StatementBlock {
    fn emit_c(&self) -> String {
        let stmts = fold_str(self.0.iter().map(|stmt| stmt.emit_c()), "\n");
        if stmts.is_empty() {
            String::from("{}")
        } else {
            format!("{{\n{stmts}\n}}\n")
        }
    }
}
impl EmitC for FunctionDefinition {
    fn emit_c(&self) -> String {
        let decl = self.declaration.emit_c();
        let body = self.body.emit_c();
        format!("{decl} {body}")
    }
}

impl EmitC for Reassignment {
    fn emit_c(&self) -> String {
        format!("{} = {}", self.assignee, self.value.emit_c())
    }
}

macro_rules! set {
        () => {{use std::collections::HashSet;HashSet::new()}};
        ($($e:expr),+) => {{
            use std::collections::HashSet;
            HashSet::from_iter(vec![$($e),+])
        }}
    }

#[cfg(test)]
mod tests {
    use crate::{fold_str, EmitC};
    use zea_ast::c;
    use zea_ast::c::datatype::{TypeQualifier, TypeSpecifier};

    #[test]
    fn test_canonicalize_zea_identifier() {
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
    fn test_fold_str() {
        let v = vec!["a", "b", "c"].into_iter().map(String::from);
        assert_eq!("a b c", fold_str(v.clone(), " "));
        assert_eq!("a,b,c", fold_str(v.clone(), ","));
        let v = vec!["a"].into_iter().map(String::from);
        assert_eq!("a", fold_str(v.clone(), " "));
        assert_eq!("a", fold_str(v.clone(), "asvbsibrburbvuorbvourbv"));
    }

    #[test]
    fn test_decl_emit_c() {
        let declb = c::statement::VariableDeclaration {
            typ: c::Type {
                qualifiers: set![],
                specifier: TypeSpecifier::Basic("int".to_string()),
            },
            name: "cat".to_string(),
        };
        assert_eq!("int cat;", declb.emit_c());

        let declb = c::statement::VariableDeclaration {
            typ: c::Type {
                qualifiers: set![TypeQualifier::Static],
                specifier: TypeSpecifier::Basic("int".to_string()),
            },
            name: "cat".to_string(),
        };
        assert_eq!("static int cat;", declb.emit_c());

        let declb = c::statement::VariableDeclaration {
            typ: c::Type {
                qualifiers: set![],
                specifier: TypeSpecifier::Pointer(Box::new(TypeSpecifier::Basic(
                    "int".to_string(),
                ))),
            },
            name: "cat".to_string(),
        };
        assert_eq!("int *cat;", declb.emit_c());

        let declb = c::statement::VariableDeclaration {
            typ: c::Type {
                qualifiers: set![TypeQualifier::Static],
                specifier: TypeSpecifier::Pointer(Box::new(TypeSpecifier::Basic(
                    "int".to_string(),
                ))),
            },
            name: "cat".to_string(),
        };
        assert_eq!("static int *cat;", declb.emit_c());
    }

    fn test_init_emit_c() {
        let initc = c::statement::Initialisation {
            typ: c::Type {
                qualifiers: set![TypeQualifier::Static],
                specifier: TypeSpecifier::Basic("int".to_string()),
            },
            name: "bob".to_string(),
            value: c::Expression::Literal(c::expression::Literal::Integer(3)),
        };
    }
}
