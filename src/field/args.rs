use crate::field::visibility::Visibility;
use std::convert::{TryFrom, TryInto};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Meta, NestedMeta};

/// The arguments within the `#[set(...)]` field attribute
#[derive(Debug, Default, PartialEq)]
pub struct Args {
    pub into: bool,
    pub strip: bool,
    pub visibility: Visibility,
}

impl<'a> TryFrom<&'a Attribute> for Args {
    type Error = FromAttributeError;

    fn try_from(attribute: &Attribute) -> Result<Self, Self::Error> {
        if attribute.path.is_ident("set") {
            let arguments = attribute
                .parse_meta()
                .map_err(|_| FromAttributeError::MalformedAttribute)?;
            if let Meta::List(list) = arguments {
                Ok((&list.nested).try_into()?)
            } else if let Meta::Path(_) = arguments {
                Ok(Self::default())
            } else {
                Err(FromAttributeError::MalformedAttribute)
            }
        } else {
            Err(FromAttributeError::UnrecognisedAttribute)
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FromAttributeError {
    #[error("malformed attribute")]
    MalformedAttribute,

    #[error("unrecognised attribute")]
    UnrecognisedAttribute,

    #[error(transparent)]
    FromPunctuated(#[from] FromPunctuatedError),
}

impl<'a> TryFrom<&'a Punctuated<NestedMeta, Comma>> for Args {
    type Error = FromPunctuatedError;

    fn try_from(input: &Punctuated<NestedMeta, Comma>) -> Result<Self, Self::Error> {
        let mut args = Self::default();

        for nested_meta in input {
            let meta = if let NestedMeta::Meta(meta) = nested_meta {
                meta
            } else {
                return Err(FromPunctuatedError::UnrecognisedArg);
            };

            if parse_nested_ident(meta, "into") {
                try_set_bool(&mut args.into)?;
            } else if parse_nested_ident(meta, "strip") {
                try_set_bool(&mut args.strip)?;
            } else if let Some(Ok(visibility)) = Visibility::from_meta(meta) {
                args.visibility = visibility;
            } else {
                return Err(FromPunctuatedError::UnrecognisedArg);
            }
        }

        Ok(args)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FromPunctuatedError {
    #[error("duplicate arguments")]
    DuplicateArgs,

    #[error("unrecognised argument")]
    UnrecognisedArg,
}

fn try_set_bool(flag: &mut bool) -> Result<(), FromPunctuatedError> {
    if *flag {
        Err(FromPunctuatedError::DuplicateArgs)
    } else {
        *flag = true;
        Ok(())
    }
}

/// parse a path-like attribute
fn parse_nested_ident(meta: &Meta, ident: &str) -> bool {
    if let Meta::Path(path) = meta {
        path.is_ident(ident)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, FromAttributeError, FromPunctuatedError};
    use crate::field::visibility::Visibility;
    use std::convert::{TryFrom, TryInto};
    use syn::{
        parse::Parser, parse_quote::ParseQuote, punctuated::Punctuated, token::Comma, Attribute,
        Meta, NestedMeta,
    };
    use test_case::test_case;

    #[test]
    fn parse_pub_path() {
        let parser = Meta::parse;
        let meta = &parser.parse_str(r#"pub="crate""#).unwrap();
        if let Meta::NameValue(name_value) = meta {
            assert!(name_value.path.is_ident("pub"));
        }
    }

    #[test_case("into" => Ok(Args {into: true, strip: false, visibility: super::Visibility::private()}) ; "into")]
    #[test_case("strip" => Ok(Args {into: false, strip: true, visibility: super::Visibility::private()}) ; "strip")]
    #[test_case("into, strip" => Ok(Args {into: true, strip: true, visibility: super::Visibility::private()}) ; "into, strip")]
    #[test_case("unrecognised" => Err(FromPunctuatedError::UnrecognisedArg) ; "unrecognised argument")]
    #[test_case("into, into" => Err(FromPunctuatedError::DuplicateArgs) ; "duplicate arguments")]
    #[test_case("pub" => Ok(Args {into: false, strip: false, visibility: super::Visibility::public()}) ; "public")]
    #[test_case(r#"pub = "crate""# => Ok(Args {into: false, strip: false, visibility: super::Visibility::in_crate()}) ; "pub in crate")]
    fn parse_from_field_args(input: &str) -> Result<Args, FromPunctuatedError> {
        let parser = Punctuated::<NestedMeta, Comma>::parse_separated_nonempty;
        let args = parser.parse_str(input).unwrap();

        Args::try_from(&args)
    }

    #[test_case("#[set(into)]" => Ok(Args {into: true, strip: false, visibility: super::Visibility::private()}) ; "into")]
    #[test_case(r#"#[set(into, strip, pub = "crate")]"# => Ok(Args {into: true, strip: true, visibility: super::Visibility::in_crate()}) ; "everything")]
    #[test_case("#[det(into)]" => Err(FromAttributeError::UnrecognisedAttribute))]
    #[test_case("#[set]" => Ok(Args {into: false, strip: false, visibility: super::Visibility::private()}) ; "no args")]
    fn parse_from_field_attribute(input: &str) -> Result<Args, FromAttributeError> {
        let parser = Attribute::parse_outer;
        let attribute = &parser.parse_str(input).unwrap()[0];
        attribute.try_into()
    }
}
