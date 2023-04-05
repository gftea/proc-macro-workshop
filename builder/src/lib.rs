use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (fields, types): (Vec<_>, Vec<_>) = match input.data {
        syn::Data::Struct(ref data) => {
            let fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                (
                    quote_spanned! {field_name.span()=>
                        #field_name
                    },
                    quote_spanned! {field_type.span()=>
                        #field_type
                    },
                )
            });
            fields.unzip()
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };
    let builder_name = format_ident!("{}Builder", name);
    let mut impl_builder = quote! {
        pub struct #builder_name {
            #( #fields: Option<#types>, )*
        }
        impl #builder_name {
            #(fn #fields(&mut self, #fields: #types) -> &mut Self {
                self.#fields = Some(#fields);
                self
            })*
        }
    };
    let impl_command = quote! {

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }


    };

    impl_builder.extend(impl_command);
    impl_builder.into()
}
