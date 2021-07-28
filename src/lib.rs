#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, DeriveInput, Field, FieldsNamed, Meta, MetaList, NestedMeta};

mod field_attribute;

#[proc_macro_derive(FluentSetters, attributes(set))]
pub fn derive_fluent_setters(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    build_impl_block(ast)
}

fn build_impl_block(ast: DeriveInput) -> TokenStream {
    let struct_data = match ast.data {
        syn::Data::Struct(s) => s,
        _ => panic!("setters can only be derived for structs"),
    };

    let named_fields = match struct_data.fields {
        syn::Fields::Named(f) => f,
        _ => panic!("setters can only be derived for named fields"),
    };

    let name = ast.ident;
    let setters = build_setters(named_fields);

    let impl_block = quote! {
        impl #name {
            #setters
        }
    };

    TokenStream::from(impl_block)
}

fn build_setters(fields: FieldsNamed) -> TokenStream2 {
    fields.named.into_iter().map(build_setter).collect()
}

fn build_setter(field: Field) -> TokenStream2 {
    let name = field
        .ident
        .expect("setters can only be derived for named fields");
    let ty = field.ty;

    let attrs = find_set_attributes(&field.attrs);

    if contains_into(attrs) {
        quote! {
            fn #name(mut self, #name: impl Into<#ty>) -> Self {
                self.#name = #name.into();
                self
            }
        }
    } else {
        quote! {
            fn #name(mut self, #name: #ty) -> Self {
                self.#name = #name;
                self
            }
        }
    }
}

/// find all nested attributes inside a 'set' attribute
///
/// ie. `#[set(get, these, attributes)]`
fn find_set_attributes(attrs: &[Attribute]) -> impl Iterator<Item = NestedMeta> {
    fn find_meta_list(meta: Meta) -> Option<MetaList> {
        if let Meta::List(list) = meta {
            Some(list)
        } else {
            None
        }
    }

    attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .filter_map(find_meta_list)
        .find(|meta_list| meta_list.path.is_ident("set"))
        .map(|set_attrs| set_attrs.nested.into_iter())
        .into_iter()
        .flatten()
}

fn contains_into(mut nested_attrs: impl Iterator<Item = NestedMeta>) -> bool {
    nested_attrs.any(|a| {
        if let NestedMeta::Meta(Meta::Path(path)) = a {
            path.is_ident("into")
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;
    use syn::{
        parse::Parser, punctuated::Punctuated, token::Colon2, Attribute, Ident, Meta, NestedMeta,
        Path, PathArguments, PathSegment,
    };

    #[test]
    fn find_set_attributes_into() {
        let tokens = quote! {
            #[set(into)]
        };

        let parser = Attribute::parse_outer;
        let input = parser.parse2(tokens).unwrap();
        let output: Vec<NestedMeta> = super::find_set_attributes(&input).collect();

        let mut segments = Punctuated::<PathSegment, Colon2>::default();
        segments.push(PathSegment {
            ident: Ident::new("into", Span::call_site()),
            arguments: PathArguments::default(),
        });

        let attrs = vec![NestedMeta::Meta(Meta::Path(Path {
            leading_colon: None,
            segments,
        }))];

        assert_eq!(attrs, output);
    }
}
