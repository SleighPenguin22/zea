mod structures;

use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, ItemEnum};

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

#[proc_macro_derive(VariantToStr)]
pub fn variant_to_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as ItemEnum);
    let ident = parsed.ident.clone();
    let variant_idents: Vec<(Ident, usize)> = parsed
        .variants
        .iter()
        .map(|var| (var.ident.clone(), var.fields.len()))
        .collect();
    let variant_idents_to_str = variant_idents.iter().map(|(var_ident, len)| match len {
        0 => quote! { #ident::#var_ident => stringify!(#var_ident)},
        1 => quote! { #ident::#var_ident(_) => stringify!(#var_ident)},
        _ => quote! { #ident::#var_ident(..) => stringify!(#var_ident)},
    });

    let generics = parsed.generics.clone();

    quote! {
        impl #generics #ident #generics {
            pub fn variant_as_str(&self) -> &'static str {
                match self {
                    #(#variant_idents_to_str),*
                }
            }
        }
    }
    .into()
}
