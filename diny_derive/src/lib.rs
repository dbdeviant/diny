#![forbid(unsafe_code)]

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

#[macro_use]
mod macros;
mod data;
mod serialize;

use proc_macro2::TokenStream;

/// Generate both async serialization and deserialization code
#[proc_macro_derive(AsyncSerialization)]
pub fn derive_diny_aysnc_serialization(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_input(&parse_macro_input!(input as syn::DeriveInput))
    .map_or_else(
        |err| err,
        |def| {
            let ser = wrap_in_const_unit_block(serialize::code::generate_async_serialize  (&def));
            let de  = wrap_in_const_unit_block(serialize::code::generate_async_deserialize(&def));
        
            quote! {
                #ser
                #de
            }
        }
    )
    .into()
}

/// Generate only async serialization code
#[proc_macro_derive(AsyncSerialize)]
pub fn derive_diny_async_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_input(&parse_macro_input!(input as syn::DeriveInput))
    .map_or_else(
        |err| err,
        |def| wrap_in_const_unit_block(serialize::code::generate_async_serialize(&def))
    )
    .into()
}

/// Generate only async deserialization code
#[proc_macro_derive(AsyncDeserialize)]
pub fn derive_diny_async_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_input(&parse_macro_input!(input as syn::DeriveInput))
    .map_or_else(
        |err| err,
        |def| wrap_in_const_unit_block(serialize::code::generate_async_deserialize(&def))
    )
    .into()
}


fn parse_input(input: &syn::DeriveInput) -> Result<data::Def, TokenStream> {
    let mut errors = data::Errors::new();
    data::Def::parse_input(input, &mut errors)
    .map_or_else(
        |_| {
            let e: TokenStream = errors.into();
            Err(quote! { #e })
        },
        Ok
    )
}

fn wrap_in_const_unit_block(output: TokenStream) -> TokenStream {
    quote! {
        const _: () = {
            #output
        };
    }
}