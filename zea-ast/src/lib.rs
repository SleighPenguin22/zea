/// This crate holds the AST definition for the Zea language, along with the target C AST.
pub mod c;

pub mod zea;

pub mod helper_impls;
#[cfg(feature = "visualisation")]
pub mod visualisation;

pub trait ZeaError {
    type ErrContext;
    fn zea_error_format(&self, ctx: &Self::ErrContext) -> String;
}
