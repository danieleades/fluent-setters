use std::convert::{TryFrom, TryInto};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Lit, Meta, NestedMeta};

#[derive(Default, Debug, PartialEq, Eq)]
struct Args {
    into: bool,
    strip: bool,
    visibility: Visibility,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
    Path(String),
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
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

        for x in input {
            if parse_ident(x, "into") {
                try_set_bool(&mut args.into)?;
            } else if parse_ident(x, "strip") {
                try_set_bool(&mut args.strip)?;
            } else if parse_ident(x, "pub") {
                args.visibility = Visibility::Public;
            } else if let Some(s) = parse_name_value(x, "pub") {
                args.visibility = Visibility::Path(s);
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
fn parse_ident(meta: &NestedMeta, ident: &str) -> bool {
    if let NestedMeta::Meta(Meta::Path(path)) = meta {
        path.is_ident(ident)
    } else {
        false
    }
}

fn parse_name_value(meta: &NestedMeta, ident: &str) -> Option<String> {
    if let NestedMeta::Meta(Meta::NameValue(name_value)) = meta {
        if name_value.path.is_ident(ident) {
            if let Lit::Str(literal_string) = &name_value.lit {
                return Some(literal_string.value());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Args, FromAttributeError, FromPunctuatedError, Visibility};
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

    #[test_case("into" => Ok(Args {into: true, strip: false, visibility: Visibility::Private}) ; "into")]
    #[test_case("strip" => Ok(Args {into: false, strip: true, visibility: Visibility::Private}) ; "strip")]
    #[test_case("into, strip" => Ok(Args {into: true, strip: true, visibility: Visibility::Private}) ; "into, strip")]
    #[test_case("unrecognised" => Err(FromPunctuatedError::UnrecognisedArg) ; "unrecognised argument")]
    #[test_case("into, into" => Err(FromPunctuatedError::DuplicateArgs) ; "duplicate arguments")]
    #[test_case("pub" => Ok(Args {into: false, strip: false, visibility: Visibility::Public}) ; "public")]
    #[test_case(r#"pub = "crate""# => Ok(Args {into: false, strip: false, visibility: Visibility::Path("crate".to_string())}) ; "public crate")]
    fn parse_from_field_args(input: &str) -> Result<Args, FromPunctuatedError> {
        let parser = Punctuated::<NestedMeta, Comma>::parse_separated_nonempty;
        let args = parser.parse_str(input).unwrap();

        Args::try_from(&args)
    }

    #[test_case("#[set(into)]" => Ok(Args {into: true, strip: false, visibility: Visibility::Private}) ; "into")]
    #[test_case(r#"#[set(into, strip, pub = "crate")]"# => Ok(Args {into: true, strip: true, visibility: Visibility::Path("crate".to_string())}) ; "everything")]
    #[test_case("#[det(into)]" => Err(FromAttributeError::UnrecognisedAttribute))]
    fn parse_from_field_attribute(input: &str) -> Result<Args, FromAttributeError> {
        let parser = Attribute::parse_outer;
        let attribute = &parser.parse_str(input).unwrap()[0];
        attribute.try_into()
    }
}
