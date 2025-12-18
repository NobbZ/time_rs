// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let type_name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let quoted = quote! {
        impl #impl_generics crate::sourcing::message::Message for #type_name #ty_generics #where_clause {
            fn name(&self) -> &'static str {
                concat!(module_path!(), "::", stringify!(#type_name))
            }
        }
    };

    TokenStream::from(quoted)
}
