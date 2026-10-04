use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(FamilyMember)]
pub fn family_member(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let family_name = format_ident!("{name}Family");

    let expanded = quote! {
        pub struct #family_name;

        impl crate::monadic_type_classes::Family for #family_name {
            type Member<T> = #name<T>;
        }

        impl<T> crate::monadic_type_classes::FamilyMember<T> for #name<T> {
            type Of = #family_name;
        }
    };

    TokenStream::from(expanded)
}
