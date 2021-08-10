#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use proc_macro::TokenStream;
use syn::DeriveInput;

use data::Data;

mod data;
mod field;

#[proc_macro_derive(FluentSetters, attributes(set))]
pub fn derive_fluent_setters(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let data = Data::from_derive_input(ast);

    data.generate_impl().into()
}
