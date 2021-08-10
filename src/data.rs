use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use super::field::Field;

pub struct Data {
    name: Ident,
    generics: Generics,
    fields: Vec<Field>,
}

impl Data {
    pub fn from_derive_input(derive_input: DeriveInput) -> Self {
        if let syn::Data::Struct(data_struct) = derive_input.data {
            let name = derive_input.ident;
            let generics = derive_input.generics;
            let fields = data_struct
                .fields
                .into_iter()
                .filter_map(Field::from_field)
                .collect();
            Self {
                name,
                generics,
                fields,
            }
        } else {
            panic!("can only generate setters for structs")
        }
    }

    pub fn generate_impl(&self) -> TokenStream2 {
        if self.fields.is_empty() {
            return TokenStream2::default();
        }

        let name = &self.name;
        let generics = &self.generics;
        let setters: TokenStream2 = self.fields.iter().map(Field::generate_setter).collect();

        quote! {
            impl#generics #name#generics {
                #setters
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{
        parse::{Parse, Parser},
        DeriveInput,
    };
    use test_case::test_case;

    #[test_case(
        quote! {
            struct MyStruct {
                #[set]
                a: u32
            }
        },
        &quote! {
            impl MyStruct {
                fn a(mut self, a: u32) -> Self {
                    self.a = a;
                    self
                }
            }
        }
        ; "basic"
    )]
    #[test_case(
        quote! {
            struct MyStruct {
                #[set(into)]
                a: u32
            }
        },
        &quote! {
            impl MyStruct {
                fn a(mut self, a: impl Into<u32>) -> Self {
                    self.a = a.into();
                    self
                }
            }
        }
        ; "into"
    )]
    #[test_case(
        quote! {
            struct MyStruct {
                /// This has a comment
                #[set]
                a: u32
            }
        },
        &quote! {
            impl MyStruct {
                #[doc = " This has a comment"]
                fn a(mut self, a: u32) -> Self {
                    self.a = a;
                    self
                }
            }
        }
        ; "with comment"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set]
                a: T
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self, a: T) -> Self {
                    self.a = a;
                    self
                }
            }
        }
        ; "with generics"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set(strip)]
                a: Option<T>
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self, a: T) -> Self {
                    self.a = Some(a);
                    self
                }
            }
        }
        ; "strip option"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set(strip, into)]
                a: Option<T>
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self, a: impl Into<T>) -> Self {
                    self.a = Some(a.into());
                    self
                }
            }
        }
        ; "strip option into"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set(strip)]
                a: bool
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self) -> Self {
                    self.a = true;
                    self
                }
            }
        }
        ; "strip bool"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set(strip, into)]
                a: bool
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self) -> Self {
                    self.a = true;
                    self
                }
            }
        } => panics
        ; "strip bool into"
    )]
    #[test_case(
        quote! {
            struct MyStruct<T> {
                #[set(strip)]
                a: u32
            }
        },
        &quote! {
            impl<T> MyStruct<T> {
                fn a(mut self) -> Self {
                    self.a = true;
                    self
                }
            }
        } => panics
        ; "strip other type"
    )]
    fn parse(input: TokenStream2, expected: &TokenStream2) {
        let parser = DeriveInput::parse;
        let derive_input = parser.parse2(input).unwrap();

        let data = Data::from_derive_input(derive_input);

        assert_eq!(data.generate_impl().to_string(), expected.to_string());
    }
}
