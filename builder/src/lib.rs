use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // check mistyped inert attribute on field
    match input.data {
        syn::Data::Struct(ref data) => {
            for field in data.fields.iter() {
                for attr in field.attrs.iter() {
                    if attr.path.is_ident("builder") {
                        match attr.parse_meta().unwrap() {
                            syn::Meta::List(list) => {
                                let nested = list.nested.iter().next().unwrap();
                                match nested {
                                    syn::NestedMeta::Meta(syn::Meta::NameValue(
                                        syn::MetaNameValue {
                                            path,
                                            lit: syn::Lit::Str(lit_str),
                                            ..
                                        },
                                    )) => {
                                        if path.is_ident("each") {
                                            lit_str.value();
                                        } else {
                                            let error = syn::Error::new_spanned(
                                                list,
                                                "expected `builder(each = \"...\")`",
                                            );
                                            return error.to_compile_error().into();
                                        }
                                    }
                                    _ => {
                                        let error = syn::Error::new(
                                            nested.span(),
                                            "unrecognized attribute",
                                        );
                                        return error.to_compile_error().into();
                                    }
                                }
                            }
                            _ => {
                                let error = syn::Error::new(attr.span(), "unrecognized attribute");
                                return error.to_compile_error().into();
                            }
                        }
                    }
                }
            }
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }

    let name = &input.ident;

    // extract  all the fields
    let (fields, arg_names, types, is_optionals) = match input.data {
        syn::Data::Struct(ref data) => {
            let fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let arg_name = match field.attrs.iter().next() {
                    Some(attr) => {
                        if attr.path.is_ident("builder") {
                            Some(match attr.parse_meta().unwrap() {
                                syn::Meta::List(list) => {
                                    let nested = list.nested.iter().next().unwrap();
                                    match nested {
                                        syn::NestedMeta::Meta(syn::Meta::NameValue(
                                            syn::MetaNameValue {
                                                path,
                                                lit: syn::Lit::Str(lit_str),
                                                ..
                                            },
                                        )) => {
                                            if path.is_ident("each") {
                                                lit_str.value()
                                            } else {
                                                todo!()
                                            }
                                        }
                                        _ => todo!(),
                                    }
                                }
                                _ => todo!(),
                            })
                        } else {
                            None
                        }
                    }
                    None => None,
                };

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
                ((field_name, arg_name), (field_type, is_optional))
            });
            // recruisively deconstruct the fields into a tuple of (field_names, field_types, is_optionals)
            let (field_info, type_info): (Vec<_>, Vec<_>) = fields.unzip();
            let (types, is_optionals): (Vec<_>, Vec<_>) = type_info.into_iter().unzip();
            let (field_names, arg_names): (Vec<_>, Vec<_>) = field_info.into_iter().unzip();

            (field_names, arg_names, types, is_optionals)
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

    let vec_args = arg_names
        .iter()
        .zip(types.iter())
        .zip(fields.iter())
        .filter(|((arg, _), ff)| arg.is_some() && &ff.to_string() != arg.as_ref().unwrap());

    let (args_pair, arg_types): (Vec<_>, Vec<_>) = vec_args
        .map(|((arg, &tt), &ff)| {
            let arg = arg.as_ref().unwrap();
            let tt = match tt {
                syn::Type::Path(p) => {
                    let s = p.path.segments.iter().next().unwrap();
                    if s.ident == "Vec" {
                        match &s.arguments {
                            syn::PathArguments::AngleBracketed(a) => {
                                match a.args.iter().next().unwrap() {
                                    syn::GenericArgument::Type(t) => t,
                                    _ => todo!(),
                                }
                            }
                            _ => todo!(),
                        }
                    } else {
                        panic!("expected Vec")
                    }
                }
                _ => tt,
            };
            ((ff, format_ident!("{}", arg)), tt)
        })
        .unzip();

    let (arg_fields, arg_names): (Vec<_>, Vec<_>) = args_pair.into_iter().unzip();

    // println!(
    //     "arg fields: {:#?}, {:#?}, {:#?}",
    //     arg_fields, arg_names, arg_types,
    // );

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
            #( #fields: std::option::Option<#types>, )*
        }
        impl #builder_name {
            #(pub fn #fields(&mut self, #fields: #types) -> &mut Self {
                self.#fields = std::option::Option::Some(#fields);
                self
            })*

            #(pub fn #arg_names(&mut self, #arg_names: #arg_types) -> &mut Self {
                if (self.#arg_fields.is_none()) {
                    self.#arg_fields = std::option::Option::Some(Vec::new());
                }

                self.#arg_fields.as_mut().unwrap().push(#arg_names);
                self
            })*

            pub fn build(&mut self) -> std::result::Result<Command, std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::Ok(Command {
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
