pub use error::ZeaTypeError;

/// Errors relating to AST analysis
mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};
    pub struct ZeaTypeError(String);

    impl Into<ZeaTypeError> for String {
        fn into(self) -> ZeaTypeError {
            ZeaTypeError(self)
        }
    }

    impl Into<ZeaTypeError> for &str {
        fn into(self) -> ZeaTypeError {
            ZeaTypeError(self.into())
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
