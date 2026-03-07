/// Errors relating to AST analysis
mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};
    pub struct ZeaTypeError(String);

    impl From<String> for ZeaTypeError {
        fn from(val: String) -> Self {
            ZeaTypeError(val)
        }
    }

    impl From<&str> for ZeaTypeError {
        fn from(val: &str) -> Self {
            ZeaTypeError(val.into())
        }
    }
    impl Display for ZeaTypeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Debug for ZeaTypeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for ZeaTypeError {}

    impl ZeaTypeError {
        pub fn wrap(self, template: impl Fn(String) -> String) -> ZeaTypeError {
            ZeaTypeError(template(self.0))
        }
    }
}
