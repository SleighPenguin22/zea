mod structures;

use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, ItemStruct};

fn impl_struct(ident: Ident) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {
        impl Hash for #ident {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state)
            }
        }
    })
}

#[proc_macro_derive(HashById)]
pub fn hash_by_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as DeriveInput);
    let ident = parsed.ident;
    match parsed.data {
        Data::Struct(_) => impl_struct(ident),

        Data::Enum(_) => panic!("cannot derive HashById on a union."),
        Data::Union(_) => panic!("cannot derive HashById on a union."),
    }
}

#[proc_macro_attribute]
pub fn generator(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as ItemStruct);

    let ident = parsed.ident;
    let vis = parsed.vis;
    let expanded = quote! {
        // #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #vis struct #ident {
            cur: usize,
        }
        impl #ident {
            pub fn new() -> Self {
                Self { cur: 0}
            }
            pub fn get(&mut self) -> usize {
                let cur = self.cur;
                self.cur+=1;
                cur
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
