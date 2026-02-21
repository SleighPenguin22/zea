mod expr;
mod lowering;
mod stmt;

use crate::ast::ZeaTypeIdent;
use crate::codegen::error::CodeGenError;

mod error {
    pub struct CodeGenError(String);
    impl std::fmt::Display for CodeGenError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::fmt::Debug for CodeGenError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for CodeGenError {}
}

pub type CodegenResult<T> = Result<T, CodeGenError>;

pub trait CMiscellaneous {
    fn format(&self) -> String;
}

impl ZeaTypeIdent {
    fn format_assignee(&self, assignee: String) -> String {
        match self {
            Self::Basic(t) => assignee,
            Self::ArrayOf(t) => {
                let assignee = Self::format_array_of(assignee);
                self.format_assignee(assignee)
            }
            Self::Ptr(t) => {
                let assignee = Self::format_pointer_to(assignee);
                self.format_assignee(assignee)
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
    #[test]
    fn format_assignee() {
        use crate::ast::utils::types as ZT;

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
}
