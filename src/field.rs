pub use attributes::Attributes;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

mod args;
mod attributes;
mod ty;
mod visibility;

use ty::Type;

pub struct Field {
    name: Ident,
    ty: Type,
    attrs: Attributes,
}

impl Field {
    pub fn from_field(field: syn::Field) -> Option<Self> {
        let name = field
            .ident
            .expect("setters can only be derived for named fields");
        let ty = field.ty.into();
        let attrs = Attributes::try_from_attrs(&field.attrs).unwrap()?;

        Some(Self { name, ty, attrs })
    }

    pub fn generate_setter(&self) -> TokenStream2 {
        let doc = &self.attrs.doc_attribute();
        let visibility = &self.attrs.visibility;
        let field = &self.name;
        let ty = &self.ty;

        match (self.attrs.into, self.attrs.strip, &self.ty) {
            (true, true, Type::Bool(_)) => panic!("can't use both 'strip' and 'into' on a bool"),
            (true, true, Type::Option(option)) => {
                let inner_ty = option.inner_ty();
                quote! {
                    #doc
                    #visibility fn #field(mut self, #field: impl Into<#inner_ty>) -> Self {
                        self.#field = Some(#field.into());
                        self
                    }
                }
            }
            (_, true, Type::Other(_)) => {
                panic!("'strip' argument is only valid for `Option` and `bool` fields")
            }
            (true, false, _) => {
                quote! {
                    #doc
                    #visibility fn #field(mut self, #field: impl Into<#ty>) -> Self {
                        self.#field = #field.into();
                        self
                    }
                }
            }
            (false, true, Type::Bool(_)) => {
                quote! {
                    #doc
                    #visibility fn #field(mut self) -> Self {
                        self.#field = true;
                        self
                    }
                }
            }
            (false, true, Type::Option(option)) => {
                let inner_ty = option.inner_ty();
                quote! {
                    #doc
                    #visibility fn #field(mut self, #field: #inner_ty) -> Self {
                        self.#field = Some(#field);
                        self
                    }
                }
            }
            (false, false, _) => {
                quote! {
                    #doc
                    #visibility fn #field(mut self, #field: #ty) -> Self {
                        self.#field = #field;
                        self
                    }
                }
            }
        }
    }
}
