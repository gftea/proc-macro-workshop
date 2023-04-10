use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Attribute, GenericParam, Generics, Lit, Meta, NestedMeta, Type,
    WherePredicate,
};

fn get_fmtstr(field: &syn::Field) -> Option<String> {
    for attr in field.attrs.iter() {
        if attr.path.is_ident("debug") {
            match attr.parse_meta().unwrap() {
                syn::Meta::NameValue(nv_pair) => match nv_pair.lit {
                    syn::Lit::Str(lit_str) => {
                        return Some(lit_str.value());
                    }
                    _ => break,
                },
                _ => break,
            }
        }
    }
    None
}

// recursively check if the type is used in the type path
fn check_type_in_arguments(arguments: &syn::PathArguments, ty_param: &syn::Ident) -> bool {
    match arguments {
        syn::PathArguments::AngleBracketed(args) => {
            for arg in args.args.iter() {
                if let syn::GenericArgument::Type(ty) = arg {
                    if let Type::Path(type_path) = ty {
                        if type_path.path.is_ident(ty_param) {
                            return true;
                        } else {
                            return check_type_in_arguments(
                                &type_path.path.segments[0].arguments,
                                ty_param,
                            );
                        }
                    }
                }
            }
            return false;
        }
        syn::PathArguments::None => return false,
        syn::PathArguments::Parenthesized(_) => todo!(),
    }
}

// You can identify associated types as any syn::TypePath in which the first
// path segment is one of the type parameters and there is more than one
// segment.
fn get_where_predicate(type_path: &syn::TypePath, ty_id: &Ident) -> Option<WherePredicate> {
    if type_path.path.segments.len() > 1 {
        if type_path.path.segments[0].ident == *ty_id {
            let ty_path = &type_path.path;
            return Some(parse_quote! {
                #ty_path: std::fmt::Debug
            });
        }
    } else {
        match &type_path.path.segments[0].arguments {
            syn::PathArguments::None => return None,
            syn::PathArguments::AngleBracketed(args) => {
                for arg in args.args.iter() {
                    if let syn::GenericArgument::Type(ty) = arg {
                        if let Type::Path(type_path) = ty {
                            return get_where_predicate(type_path, ty_id);
                        }
                    }
                }
            }
            syn::PathArguments::Parenthesized(_) => todo!(),
        };
    }
    None
}
fn add_where_predicates(mut generics: Generics, field_types: &Vec<&Type>) -> Generics {
    // get all type parameters
    let type_params: Vec<Ident> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.clone()),
            _ => None,
        })
        .collect();

    for ty_param_id in type_params {
        for field_ty in field_types {
            if let Type::Path(type_path) = field_ty {
                let wp = get_where_predicate(type_path, &ty_param_id);
                if let Some(wp) = wp {
                    generics.make_where_clause().predicates.push(wp);
                }
            }
        }
    }
    generics
}
// Add a bound `T: std::fmt::Debug` to every type parameter T.
fn add_trait_bounds(mut generics: Generics, field_types: &Vec<&Type>) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            // check if the type parameter is only used in PhantomData
            for ty in field_types.iter() {
                if let Type::Path(type_path) = ty {
                    // type parameter is used in the field type directly
                    if type_path.path.is_ident(&type_param.ident) {
                        type_param.bounds.push(parse_quote!(std::fmt::Debug));
                    }
                    // type parameter is used in the field type as a generic argument
                    else if let Some(v) = type_path.path.segments.last() {
                        if v.ident != "PhantomData" {
                            let is_contained =
                                check_type_in_arguments(&v.arguments, &type_param.ident);
                            if is_contained {
                                type_param.bounds.push(parse_quote!(std::fmt::Debug));
                            }
                        }
                    }
                }
            }
        }
    }
    generics
}

fn custom_bound(mut generics: Generics, attrs: Vec<Attribute>) -> (Generics, bool) {
    let mut disable_inference_bounds = false;
    for attr in attrs.iter() {
        if attr.path.is_ident("debug") {
            let meta = attr.parse_meta().unwrap();
            if let Meta::List(list) = meta {
                for nested in list.nested.iter() {
                    if let NestedMeta::Meta(Meta::NameValue(nv_pair)) = nested {
                        if nv_pair.path.is_ident("bound") {
                            if let Lit::Str(lit_str) = &nv_pair.lit {
                                let bound = lit_str.parse::<WherePredicate>().unwrap();

                                generics.make_where_clause().predicates.push(bound);
                                disable_inference_bounds = true;
                            }
                        }
                    }
                }
            }
        }
    }
    (generics, disable_inference_bounds)
}
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    // eprintln!("INPUT: {:#?}", input);

    let type_id = &input.ident;
    match &input.data {
        syn::Data::Struct(data) => {
            let mut fields = std::vec::Vec::new();
            let mut field_types = std::vec::Vec::new();
            for field in data.fields.iter() {
                let field_name = field.ident.as_ref().unwrap();
                field_types.push(&field.ty);
                let fmtstr = get_fmtstr(field);
                match fmtstr {
                    Some(s) => fields.push(quote!(
                        stringify!(#field_name),
                        &format_args!(#s, &self.#field_name)
                    )),
                    None => fields.push(quote! {
                        stringify!(#field_name), &self.#field_name
                    }),
                }
            }
            let (mut generics, dis_inference) = custom_bound(input.generics, input.attrs);
            if !dis_inference {
                generics = add_trait_bounds(generics, &field_types);
                generics = add_where_predicates(generics, &field_types);
            }
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            let ts = quote!(
                impl #impl_generics std::fmt::Debug for #type_id #ty_generics #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(stringify!(#type_id))
                            #(.field(#fields))*
                            .finish()
                    }
                }
            );
            // eprintln!("TOKENS: {}", ts);

            return ts.into();
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}
