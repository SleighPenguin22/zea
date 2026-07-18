use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use indexmap::IndexSet;

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

#[derive(PartialEq, Eq, Clone)]
struct InternTable<Key: From<usize> + Into<usize>, Value: Hash + Eq> {
    set: IndexSet<Value>,
    phantom: PhantomData<Key>,
}

impl<Key: From<usize> + Into<usize> + Debug, Value: Hash + Eq + Debug> Debug
    for InternTable<Key, Value>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternTable")
            .field("set", &self.set)
            .finish()
    }
}

impl<Key: From<usize> + Into<usize>, Value: Hash + Eq> InternTable<Key, Value> {
    pub fn new() -> Self {
        Self {
            set: IndexSet::new(),
            phantom: PhantomData,
        }
    }
    pub fn intern(&mut self, item: Value) -> Key {
        let (idx, _) = self.set.insert_full(item);
        idx.into()
    }
    pub fn get_by_id(&self, id: Key) -> Option<&Value> {
        self.set.get_index(id.into())
    }
}
impl<Key: From<usize> + Into<usize>, Value: Hash + Eq> Default for InternTable<Key, Value> {
    fn default() -> Self {
        Self::new()
    }
}
