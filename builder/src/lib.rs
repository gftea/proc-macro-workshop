use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (fields, types): (Vec<_>, Vec<_>) = match input.data {
        syn::Data::Struct(ref data) => {
            let fields = data.fields.iter().filter_map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                let is_optional = match field_type {
                    syn::Type::Path(p) => p.path.segments.iter().any(|s| s.ident == "Option"),
                    _ => todo!(),
                };
                if is_optional {
                    return None;
                }
                Some((
                    quote_spanned! {field_name.span()=>
                        #field_name
                    },
                    quote_spanned! {field_type.span()=>
                        #field_type
                    },
                ))
            });
            fields.unzip()
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };
    let (optional_fields, optional_types): (Vec<_>, Vec<_>) = match input.data {
        syn::Data::Struct(ref data) => {
            let fields = data.fields.iter().filter_map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let is_optional = match field_type {
                    syn::Type::Path(p) => p.path.segments.iter().any(|s| s.ident == "Option"),
                    _ => todo!(),
                };
                if !is_optional {
                    return None;
                }
                let type_in_option = match field_type {
                    syn::Type::Path(p) => {
                        let s = p
                            .path
                            .segments
                            .iter()
                            .find(|s| s.ident == "Option")
                            .unwrap();

                        match &s.arguments {
                            syn::PathArguments::AngleBracketed(a) => {
                                match a.args.iter().next().unwrap() {
                                    syn::GenericArgument::Type(t) => t,
                                    _ => todo!(),
                                }
                            }
                            _ => todo!(),
                        }
                    }
                    _ => todo!(),
                };

                Some((
                    quote_spanned! {field_name.span()=>
                        #field_name
                    },
                    quote_spanned! {field_type.span()=>
                        #type_in_option
                    },
                ))
            });
            fields.unzip()
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };

    // `CommandBuilder` is the name of the builder struct.
    let builder_name = format_ident!("{}Builder", name);
    let mut impl_builder = quote! {
        pub struct #builder_name {
            #( #fields: Option<#types>, )*
            #( #optional_fields: Option<#optional_types>, )*
        }
        impl #builder_name {
            #(fn #fields(&mut self, #fields: #types) -> &mut Self {
                self.#fields = Some(#fields);
                self
            })*

            #(fn #optional_fields(&mut self, #optional_fields: #optional_types) -> &mut Self {
                self.#optional_fields = Some(#optional_fields);
                self
            })*

            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(Command {
                    #(#fields: self.#fields.to_owned().ok_or(
                        format!("missing field `{}`", stringify!(#fields))
                    )?,)*
                    #(#optional_fields: self.#optional_fields.to_owned(),)*
                })
            }
        }
    };
    // `Command` is the name of the struct we are deriving for.
    let impl_command = quote! {

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#fields: None,)*
                    #(#optional_fields: None,)*
                }
            }
        }
    };

    impl_builder.extend(impl_command);
    impl_builder.into()
}
