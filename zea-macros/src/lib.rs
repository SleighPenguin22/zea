mod structures;

use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput};

fn impl_struct(ident: Ident) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {
        impl PartialEq for #ident {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        impl Eq for #ident {}
        
        impl Hash for #ident {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state)
            }
        }
    })
}

#[proc_macro_derive(HashEqById)]
pub fn hash_by_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as DeriveInput);
    let ident = parsed.ident;
    match parsed.data {
        Data::Struct(_) => impl_struct(ident),

        Data::Enum(_) => panic!("cannot derive HashById on a union."),
        Data::Union(_) => panic!("cannot derive HashById on a union."),
    }
}