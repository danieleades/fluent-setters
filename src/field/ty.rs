use quote::ToTokens;
use syn::{GenericArgument, PathArguments};

#[derive(Debug)]
pub enum Type {
    Bool(syn::Type),
    Option(OptionTy),
    Other(syn::Type),
}

impl From<syn::Type> for Type {
    fn from(ty: syn::Type) -> Self {
        if let syn::Type::Path(type_path) = &ty {
            if type_path.path.segments.first().unwrap().ident == "Option" {
                return Type::Option(OptionTy { ty });
            } else if type_path.path.is_ident("bool") {
                return Type::Bool(ty);
            }
        }

        Self::Other(ty)
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = match self {
            Type::Bool(t) | Type::Option(OptionTy { ty: t }) | Type::Other(t) => t,
        };

        ty.to_tokens(tokens);
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct OptionTy {
    ty: syn::Type,
}

impl OptionTy {
    pub fn inner_ty(&self) -> &syn::Type {
        if let syn::Type::Path(type_path) = &self.ty {
            if let PathArguments::AngleBracketed(arguments) =
                &type_path.path.segments.first().unwrap().arguments
            {
                if let GenericArgument::Type(ty) = arguments.args.first().unwrap() {
                    return ty;
                }
            }
        }

        panic!()
    }
}

#[cfg(test)]
mod tests {

    use super::Type;
    use syn::parse::{Parse, Parser};
    use test_case::test_case;

    fn parse_input(input: &str) -> Type {
        let parser = syn::Type::parse;
        parser.parse_str(input).unwrap().into()
    }

    #[test_case("u32" => "other")]
    #[test_case("Option<u32>" => "option")]
    #[test_case("bool" => "bool")]
    fn parse(input: &str) -> &str {
        match parse_input(input) {
            Type::Bool(_) => "bool",
            Type::Option(_) => "option",
            Type::Other(_) => "other",
        }
    }
}
