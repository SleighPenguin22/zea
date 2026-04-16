mod structures;

use proc_macro::TokenStream;
use std::fmt::format;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Generics, ItemEnum};

fn hash_eq_by_id_impl_struct(ident: Ident) -> proc_macro::TokenStream {
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
        Data::Struct(_) => hash_eq_by_id_impl_struct(ident),

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

fn derive_ast_structural_eq_impl_struct(
    data_struct: DataStruct,
    ident: Ident,
    generics: Generics,
) -> TokenStream {
    let applicable_fields =
        data_struct
            .fields
            .iter()
            .filter_map(|field| match field.ident.clone() {
                Some(ident) if ident != "id" => Some(ident),
                _ => None,
            });
    let self_eq_other_field: Vec<_> = applicable_fields
        .map(|field_name| {
            quote! {(self.#field_name) == (other.#field_name)}
        })
        .collect();

    quote! {
        impl #generics #ident #generics {
            pub fn structural_eq(&self, other: &Self) -> bool {
                let mut is_eq = true;
                #(is_eq |= #self_eq_other_field;)*
                is_eq
            }
        }
    }
    .into()
}

fn derive_ast_structural_eq_impl_enum(data_enum: DataEnum, ident: Ident, generics: Generics) -> TokenStream {
    let arms = data_enum.variants.iter().map(|variant|
        {
            let variant_name = variant.ident.clone();
            let (self_items, other_items): (Vec<_>, Vec<_>) = (0..variant.fields.len()).map(|i| (format!("s{}", i),format!("o{}", i))).unzip();
            quote! {
                (#variant_name(#(self_items)),#variant_name(other_items))if #()
            }
        })
    
    
    quote! {
        impl #generics #ident #generics {
            pub fn structural_eq(&self, other: &Self) -> bool {
                self.eq(other)
            }
        }
    }
    .into()
}

#[proc_macro_derive(ASTStructuralEq)]
pub fn derive_ast_structural_eq(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as DeriveInput);
    let ident = parsed.ident.clone();
    let generics = parsed.generics.clone();
    match parsed.data {
        Data::Struct(s) => derive_ast_structural_eq_impl_struct(s, ident, generics),
        Data::Enum(e) => derive_ast_structural_eq_impl_enum(ident, generics),
        Data::Union(_) => panic!("strucutral equality on Unions is not supported"),
    }
}
