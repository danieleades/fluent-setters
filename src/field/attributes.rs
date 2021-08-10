use crate::field::args::{Args, FromAttributeError};
use quote::quote;
use std::convert::TryFrom;
use syn::{parse::Parser, Attribute, Lit, Meta, MetaNameValue};

use super::visibility::Visibility;

/// The full set of possible field attributes that this crate is interested in
#[derive(Default, Debug, PartialEq)]
pub struct Attributes {
    pub into: bool,
    pub strip: bool,
    pub visibility: Visibility,
    pub doc: Option<String>,
}

impl Attributes {
    pub fn try_from_attrs(attrs: &[Attribute]) -> Result<Option<Self>, FromAttributeError> {
        let mut set = None;
        let mut doc = None;

        for attr in attrs.iter() {
            if let Ok(a) = Args::try_from(attr) {
                set = Some(a);
            } else if let Ok(Meta::NameValue(name_value)) = attr.parse_meta() {
                match parse_doc(&name_value) {
                    Some(Ok(s)) => doc = Some(s),
                    Some(Err(())) => return Err(FromAttributeError::MalformedAttribute),
                    None => (),
                }
            }
        }

        Ok(set.map(|set| Self {
            into: set.into,
            strip: set.strip,
            visibility: set.visibility,
            doc,
        }))
    }

    pub fn doc_attribute(&self) -> Option<Attribute> {
        let doc = self.doc.as_ref()?;
        let parser = Attribute::parse_outer;
        let tokens = quote! {
            #[doc = #doc]
        };
        Some(parser.parse2(tokens).unwrap().into_iter().next().unwrap())
    }
}

fn parse_doc(name_value: &MetaNameValue) -> Option<Result<String, ()>> {
    if !name_value.path.is_ident("doc") {
        None
    } else if let Lit::Str(lit_str) = &name_value.lit {
        Some(Ok(lit_str.value()))
    } else {
        Some(Err(()))
    }
}

#[cfg(test)]
mod tests {
    use super::Attributes;
    use crate::field::visibility::Visibility;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{parse::Parser, Field};
    use test_case::test_case;

    #[test_case(
        quote!{
            #[set(into, strip)]
            some_field: String
        }
        => Attributes {into: true, strip: true, visibility: super::Visibility::private(), doc: None}
        ; "plain attributes"
    )]
    #[test_case(
        quote!{
            /// This is a doc comment
            #[set(into, strip)]
            some_field: String
        }
        => Attributes {into: true, strip: true, visibility: super::Visibility::private(), doc: Some(" This is a doc comment".to_string())}
        ; "with doc comment"
    )]
    #[test_case(
        quote!{
            /// This is a doc comment
            #[set(into, strip)]
            #[unrelated]
            some_field: String
        }
        => Attributes {into: true, strip: true, visibility: super::Visibility::private(), doc: Some(" This is a doc comment".to_string())}
        ; "unrelated attributes"
    )]
    #[test_case(
        quote!{
            #[set]
            some_field: String
        }
        => Attributes {into: false, strip: false, visibility: super::Visibility::private(), doc: None}
        ; "empty attributes"
    )]
    fn from_attributes(tokens: TokenStream2) -> Attributes {
        let parser = Field::parse_named;
        let raw_attrs = parser.parse2(tokens).unwrap().attrs;

        Attributes::try_from_attrs(&raw_attrs).unwrap().unwrap()
    }
}
