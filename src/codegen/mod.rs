mod error;
mod expr;
mod lowering;
mod stmt;

use crate::ast::ZeaTypeIdent;
use crate::codegen::error::CodeGenError;

pub type CodegenResult<T> = Result<T, CodeGenError>;

impl ZeaTypeIdent {
    fn format_assignee(&self, assignee: String) -> String {
        match self {
            Self::Basic(_t) => assignee,
            Self::ArrayOf(t) => {
                let assignee = Self::format_array_of(assignee);
                t.format_assignee(assignee)
            }
            Self::Ptr(t) => {
                let assignee = Self::format_pointer_to(assignee);
                t.format_assignee(assignee)
            }
            Self::Slice(_t) => unimplemented!("slices are not yet supported"),
            Self::Option(_t) => unimplemented!("optional types are not yet supported"),
        }
    }

    fn format_pointer_to(formatted_assignee: String) -> String {
        "*".to_string() + &formatted_assignee
    }

    fn format_array_of(formatted_assignee: String) -> String {
        formatted_assignee + "[]"
    }
}

mod tests {
    use super::lowering::{LowerInto, LoweredStatement, LoweredVarDecl, LoweredVarDeclAssignment};
    use crate::ast::utils::expressions as ZE;
    use crate::ast::utils::literals as ZL;
    use crate::ast::utils::statements as ZS;
    use crate::ast::utils::types as ZT;

    #[test]
    fn format_assignee() {
        let int = ZT::basic_int();
        let intptr = ZT::ptr_to(int.clone());
        let intarr = ZT::array_of(int.clone());
        let intarrptr = ZT::array_of(ZT::ptr_to(int.clone()));
        let intptrarrptr = ZT::ptr_to(ZT::array_of(ZT::ptr_to(int.clone())));

        assert_eq!(int.format_assignee("a".into()), "a");
        assert_eq!(intptr.format_assignee("a".into()), "*a");
        assert_eq!(intarr.format_assignee("a".into()), "a[]");
        assert_eq!(intarrptr.format_assignee("a".into()), "*a[]");
        assert_eq!(intptrarrptr.format_assignee("a".into()), "**a[]");
    }

    fn lower_basic_declaration() {
        let typ = ZT::basic_int();
        let value = ZE::expr_literal_int(3);
        let assign = ZS::basic_assignment_mut(typ.clone(), "a", value.clone());

        let lowered = assign
            .clone()
            .lower_into()
            .expect(&format!("should be able to lower statement {:?}", assign));

        assert_eq!(
            lowered,
            vec![LoweredStatement::DeclAssign(LoweredVarDeclAssignment {
                lowered_var_decl: LoweredVarDecl {
                    mutable: true,
                    typ,
                    assignee: "a".to_string(),
                },
                value,
            })]
        )
    }
}
