use quote::ToTokens;
use syn::{
    token::{Crate, Pub},
    Lit, Meta, MetaNameValue, VisCrate, VisPublic,
};

#[derive(Debug, PartialEq)]
pub struct Visibility(syn::Visibility);

impl Visibility {
    pub fn public() -> Self {
        Self(syn::Visibility::Public(VisPublic {
            pub_token: Pub::default(),
        }))
    }

    pub fn private() -> Self {
        Self(syn::Visibility::Inherited)
    }

    pub fn in_crate() -> Self {
        Self(syn::Visibility::Crate(VisCrate {
            crate_token: Crate::default(),
        }))
    }

    pub fn from_meta(meta: &Meta) -> Option<Result<Self, ()>> {
        if !meta.path().is_ident("pub") {
            return None;
        }

        match meta {
            Meta::Path(_) => Some(Ok(Self::public())),
            Meta::List(_) => Some(Err(())),
            Meta::NameValue(name_value) => Some(parse_restricted(name_value)),
        }
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::private()
    }
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

fn parse_restricted(input: &MetaNameValue) -> Result<Visibility, ()> {
    if let Lit::Str(lit_str) = &input.lit {
        if lit_str.value() == "crate" {
            return Ok(Visibility::in_crate());
        }
    }

    Err(())
}

#[cfg(test)]
mod tests {
    use super::Visibility;
    use syn::{
        parse::{Parse, Parser},
        Meta,
    };
    use test_case::test_case;

    #[test_case("pub" => Some(Ok(Visibility::public())) ; "public")]
    #[test_case(r#"pub = "crate""# => Some(Ok(Visibility::in_crate())) ; "pub in crate")]
    fn parse(input: &str) -> Option<Result<Visibility, ()>> {
        let parser = Meta::parse;
        let meta = parser.parse_str(input).unwrap();

        Visibility::from_meta(&meta)
    }
}
