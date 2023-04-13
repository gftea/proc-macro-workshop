use std::{collections::BTreeMap, iter::zip};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, Item, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    // eprintln!("args: {:#?}", args);
    // eprintln!("input: {:#?}", input);

    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as syn::Item);

    let item = match parse_enum_item(input) {
        Ok(item) => {
            let mut sorted_variants = item.variants.clone().into_iter().collect::<Vec<_>>();
            sorted_variants.sort_by(|a, b| a.ident.cmp(&b.ident));

            let pairs = zip(sorted_variants, &item.variants);

            for (sorted_v, ori_v) in pairs {
                if sorted_v.ident != ori_v.ident {
                    let err = syn::Error::new(
                        sorted_v.ident.span(),
                        format!("{} should sort before {}", sorted_v.ident, ori_v.ident),
                    )
                    .to_compile_error();
                    return quote!( #item #err).into();
                }
            }
            item
        }
        Err(err) => return err.to_compile_error().into(),
    };

    quote!(#item).into()
}

fn parse_enum_item(item: Item) -> syn::Result<ItemEnum> {
    match item {
        Item::Enum(item) => Ok(item),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        )),
    }
}

////////////////////////////////////////////////////////////////////////
#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    // eprintln!("args: {:#?}", args);
    // eprintln!("input: {:#?}", input);

    let mut item_fn = parse_macro_input!(input as syn::ItemFn);
    let mut visitor = SortedMatchExpr {
        ts: proc_macro2::TokenStream::new(),
    };
    syn::visit_mut::visit_item_fn_mut(&mut visitor, &mut item_fn);
    let err = visitor.ts;
    quote!(#item_fn #err).into()
}

struct SortedMatchExpr {
    ts: proc_macro2::TokenStream,
}

impl syn::visit_mut::VisitMut for SortedMatchExpr {
    fn visit_expr_match_mut(&mut self, expr: &mut syn::ExprMatch) {
        // eprintln!("match expr: {:#?}", expr);
        if expr.attrs.iter().any(|attr| attr.path.is_ident("sorted")) {
            let mut sorted_arms = expr.arms.clone().into_iter().collect::<Vec<_>>();
            sorted_arms.sort_by(|a, b| {
                if let syn::Pat::TupleStruct(a_pat) = &a.pat {
                    if let syn::Pat::TupleStruct(b_pat) = &b.pat {
                        let a_pat_ident = &a_pat.path.segments.last().unwrap().ident;
                        let b_pat_ident = &b_pat.path.segments.last().unwrap().ident;

                        return a_pat_ident.cmp(&b_pat_ident);
                    }
                }
                std::cmp::Ordering::Equal
            });

            let pairs = zip(sorted_arms, &expr.arms);

            for (sorted_a, ori_a) in pairs {
                // eprintln!("sorted_a: {:?}", sorted_a);
                // eprintln!("ori_a: {:?}", ori_a);
                if let syn::Pat::TupleStruct(sorted_pat) = &sorted_a.pat {
                    if let syn::Pat::TupleStruct(ori_pat) = &ori_a.pat {
                        let sorted_pat_ident = &sorted_pat.path.segments.last().unwrap().ident;
                        let ori_pat_ident = &ori_pat.path.segments.last().unwrap().ident;

                        let sorted_pat_path = &sorted_pat
                            .path
                            .segments
                            .iter()
                            .map(|seg| seg.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::");

                        let ori_pat_path = &ori_pat
                            .path
                            .segments
                            .iter()
                            .map(|seg| seg.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::");
                        if sorted_pat_ident != ori_pat_ident {
                            let err = syn::Error::new_spanned(
                                &sorted_pat.path,
                                format!("{} should sort before {}", sorted_pat_path, ori_pat_path),
                            )
                            .to_compile_error();
                            self.ts = err;
                            break;
                        }
                    }
                }
                match &sorted_a.pat {
                    syn::Pat::Ident(_)
                    | syn::Pat::Path(_)
                    | syn::Pat::Struct(_)
                    | syn::Pat::TupleStruct(_)
                    | syn::Pat::Wild(_) => {}
                    _ => {
                        let err =
                            syn::Error::new_spanned(&sorted_a.pat, "unsupported by #[sorted]")
                                .to_compile_error();
                        self.ts = err;
                        break;
                    }
                }
            }
            expr.attrs.retain(|attr| !attr.path.is_ident("sorted"));
        }
        syn::visit_mut::visit_expr_match_mut(self, expr);
    }
}
