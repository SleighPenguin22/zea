use crate::{ConvertC, EmitC};
use zea_ast::c;
use zea_ast::zea::lowering;
use zea_ast::zea::lowering::{ExpandedExpression, ExpandedReassignment};

/// vec![], but it boxes each of its expressions
macro_rules! vecboxed {
    ($($e:expr),*) => {
        vec![$(Box::new($e)),*]
    };
}

impl ConvertC for ExpandedReassignment {
    fn convert_c(self) -> Vec<Box<dyn EmitC>> {
        vecboxed![c::Reassignment {
            assignee: self.assignee,
            value: self.value.into(),
        }]
    }
}

impl ConvertC for ExpandedExpression {
    fn convert_c(self, ) -> Vec<Box<dyn EmitC>> {
        vecboxed![match self {
            ExpandedExpression::Ident(i) => c::Expression::Ident(i),
            _ => todo!("cannot yet convert expanded node\n{self:?}\nto C"),
        }]
    }
}

impl From<lowering::ExpandedStatement> for c::Statement {
    fn from(value: ExpandedStatement) -> Self {
        match value {
            ExpandedStatement::Return(expr) => c::Statement::Return(expr.into()),
            ExpandedStatement::Reassignment(reassign) => {
                c::Statement::Reassignment(reassign.into())
            }
            ExpandedStatement::Initialisation(init) => {
                c::Statement::VariableInitialisation(init.into())
            }
            _ => todo!("cannot yet c convert statement\n{value:?}\n"),
        }
    }
}

impl From<zea::Type> for c::TypeSpecifier {
    fn from(value: Type) -> Self {
        match value {
            Type::Basic(t) => c::TypeSpecifier::Basic(t),
            Type::Pointer(t) => c::TypeSpecifier::Pointer(Box::new(t.as_ref().clone().into())),
            Type::ArrayOf(t) => c::TypeSpecifier::Pointer(Box::new(t.as_ref().clone().into())),
        }
    }
}

impl ExpandedInitialisation {
    pub fn c_expand(self) -> Vec<c::Initialisation> {
        let temp = self.temporary;
        let spec: c::TypeSpecifier = temp
            .typ
            .expect("type of {self:?} should be known at this point")
            .into();
        let typ: c::Type = spec.into();
        let mut first = vec![c::Initialisation {
            typ,
            name: temp.assignee,
            value: temp.value.into(),
        }];
        let children: Vec<_> = self
            .unpacked_assignments
            .into_iter()
            .flat_map(|init| init.c_expand())
            .collect();
        first.extend(children);
        first
    }
}

impl From<lowering::ExpandedInitialisation> for Vec<c::Initialisation> {
    fn from(value: ExpandedInitialisation) -> Self {
        value.c_expand()
    }
}
