/// This package holds the AST nodes for the Zea language, along with the target C AST.
pub mod c;

/// This module contains the AST definition for the Zea language.
/// Any node that encompasses some structure with meaningful data has an id, this id has the following guarantees:
/// - the id is unique
/// - there is no specified order in the id's of nodes.
/// - the ID of a node stays the same through the whole compilation process.
///
/// As such, you can use these id's as keys in hashtables tables that annotate nodes.
pub mod zea;

pub mod helper_impls;
#[cfg(feature = "visualisation")]
pub mod visualisation;

pub trait StructuralEq {
    fn structural_eq(&self, other: &Self) -> bool;
}
