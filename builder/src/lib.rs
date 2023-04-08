use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // extract  all the fields
    let (fields, types, is_optionals) = match input.data {
        syn::Data::Struct(ref data) => {
            let fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                // check if the field is optional
                let (field_type, is_optional) = match field_type {
                    syn::Type::Path(p) => {
                        let s = p.path.segments.iter().next().unwrap();

                        if s.ident == "Option" {
                            match &s.arguments {
                                syn::PathArguments::AngleBracketed(a) => {
                                    match a.args.iter().next().unwrap() {
                                        syn::GenericArgument::Type(t) => (t, true),
                                        _ => todo!(),
                                    }
                                }
                                _ => todo!(),
                            }
                        } else {
                            (field_type, false)
                        }
                    }
                    _ => todo!(),
                };

                // each field is deconstructed into a tuple of (field_name, (field_type, is_optional))
                (
                    quote_spanned!(field_name.span()=>
                        #field_name
                    ),
                    (
                        quote_spanned!(field_type.span()=>
                            #field_type
                        ),
                        is_optional,
                    ),
                )
            });
            // recruisively deconstruct the fields into a tuple of (field_names, field_types, is_optionals)
            let (field_names, type_info): (Vec<_>, Vec<_>) = fields.unzip();
            let (types, is_optionals): (Vec<_>, Vec<_>) = type_info.into_iter().unzip();
            (field_names, types, is_optionals)
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };

    let mandatory_fields = fields
        .iter()
        .zip(is_optionals.iter())
        .filter(|(_, is_optional)| !**is_optional)
        .map(|(field, _)| field);

    let optional_fields = fields
        .iter()
        .zip(is_optionals.iter())
        .filter(|(_, is_optional)| **is_optional)
        .map(|(field, _)| field);

    // `CommandBuilder` is the name of the builder struct.
    let builder_name = format_ident!("{}Builder", name);

    // impl Command
    let impl_command = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#fields: None,)*
                }
            }
        }
    };
    // impl CommandBuilder
    let mut impl_builder = quote! {
        pub struct #builder_name {
            #( #fields: Option<#types>, )*
        }
        impl #builder_name {
            #(fn #fields(&mut self, #fields: #types) -> &mut Self {
                self.#fields = Some(#fields);
                self
            })*

            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(Command {
                    #(#mandatory_fields: self.#mandatory_fields.to_owned().ok_or(
                        format!("missing field `{}`", stringify!(#mandatory_fields))
                    )?,)*
                    #(#optional_fields: self.#optional_fields.to_owned(),)*
                })
            }
        }
    };

    // combine the two impls tokenstreams
    impl_builder.extend(impl_command);
    impl_builder.into()
}
