mod structures;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Generics, ItemEnum, Variant};

fn hash_eq_by_id_impl_struct(ident: Ident) -> TokenStream {
    TokenStream::from(quote! {
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
pub fn hash_by_id(input: TokenStream) -> TokenStream {
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

fn structural_eq_impl_struct(
    data_struct: DataStruct,
    ident: Ident,
    generics: Generics,
) -> TokenStream {
    let field_comparisons = build_structural_eq_struct_field_comparisons(&data_struct);

    quote! {
        impl #generics StructuralEq for #ident #generics {
            fn eq_ignore_id(&self, other: &Self) -> bool {
                let mut is_eq = true;
                #field_comparisons
                is_eq
            }
        }
    }
    .into()
}

fn build_structural_eq_struct_field_comparisons(
    data_struct: &DataStruct,
) -> proc_macro2::TokenStream {
    let applicable_fields: Vec<_> = data_struct
        .fields
        .iter()
        .filter_map(|field| field.ident.clone())
        .filter(|ident| ident != "id")
        .collect();

    let comparisons: Vec<_> = applicable_fields
        .iter()
        .map(|field_name| {
            quote! { is_eq &= (self.#field_name).eq_ignore_id(&other.#field_name); }
        })
        .collect();

    quote! { #(#comparisons)* }
}

fn structural_eq_impl_enum(data_enum: DataEnum, ident: Ident, generics: Generics) -> TokenStream {
    let arms: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| build_structural_eq_enum_variant_arm(&ident, variant))
        .collect();

    quote! {
        impl #generics StructuralEq for #ident #generics {
            fn eq_ignore_id(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#arms)*
                    _ => false
                }
            }
        }
    }
    .into()
}

fn build_structural_eq_enum_variant_arm(
    enum_ident: &Ident,
    variant: &Variant,
) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    let field_count = variant.fields.len();
    let (self_items, other_items) = build_enum_variant_subitems_idents(field_count);
    let pattern = build_structural_eq_variant_match_pattern(
        enum_ident,
        variant_name,
        field_count,
        &self_items,
        &other_items,
    );
    let equality_checks = build_field_equality_checks(&self_items, &other_items);

    quote! {
        #pattern
        if {
            let mut sub_items_eq = true;
            #equality_checks
            sub_items_eq
        } => true,
    }
}

fn build_enum_variant_subitems_idents(field_count: usize) -> (Vec<Ident>, Vec<Ident>) {
    (0..field_count)
        .map(|i| Ident::new(&format!("sf{}", i), Span::call_site()))
        .zip((0..field_count).map(|i| Ident::new(&format!("of{}", i), Span::call_site())))
        .unzip()
}

fn build_structural_eq_variant_match_pattern(
    enum_ident: &Ident,
    variant_name: &Ident,
    field_count: usize,
    self_items: &[Ident],
    other_items: &[Ident],
) -> proc_macro2::TokenStream {
    if field_count == 0 {
        quote! { (#enum_ident::#variant_name, #enum_ident::#variant_name) }
    } else {
        quote! {
            (
                #enum_ident::#variant_name(#(#self_items),*),
                #enum_ident::#variant_name(#(#other_items),*)
            )
        }
    }
}

fn build_field_equality_checks(
    self_items: &[Ident],
    other_items: &[Ident],
) -> proc_macro2::TokenStream {
    let checks: Vec<_> = self_items
        .iter()
        .zip(other_items.iter())
        .map(|(s, o)| quote! { sub_items_eq &= #s.eq_ignore_id(#o); })
        .collect();

    quote! { #(#checks)* }
}

#[proc_macro_derive(ASTStructuralEq)]
pub fn derive_ast_structural_eq(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as DeriveInput);
    let ident = parsed.ident.clone();
    let generics = parsed.generics.clone();
    match parsed.data {
        Data::Struct(s) => structural_eq_impl_struct(s, ident, generics),
        Data::Enum(e) => structural_eq_impl_enum(e, ident, generics),
        Data::Union(_) => panic!("structural equality on Unions is not supported"),
    }
}
