use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{braced, parse::Parse, parse_macro_input, punctuated::Punctuated};

#[derive(Debug)]
#[allow(dead_code)]
struct Seq {
    ident: syn::Ident,
    start: syn::LitInt,
    end: syn::LitInt,

    code_block: Punctuated<proc_macro2::TokenStream, syn::Token![;]>,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start: syn::LitInt = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let end: syn::LitInt = input.parse()?;

        let code_block;
        braced!(code_block in input);

        let code_block = code_block.parse_terminated::<proc_macro2::TokenStream, syn::Token![;]>(
            proc_macro2::TokenStream::parse,
        )?;
        Ok(Seq {
            ident,
            start,
            end,
            code_block,
        })
    }
}

fn update_codeblock(
    code_block: proc_macro2::TokenStream,
    ident: &syn::Ident,
    i: usize,
) -> proc_macro2::TokenStream {
    let mut new_code_block = proc_macro2::TokenStream::new();
    for cb in code_block {
        match cb {
            proc_macro2::TokenTree::Group(grp) => {
                let new_grp = update_codeblock(grp.stream(), ident, i);
                let mut new_grp = proc_macro2::Group::new(grp.delimiter(), new_grp);
                // retain the orignal span
                new_grp.set_span(grp.span());
                new_code_block.extend(new_grp.to_token_stream());
            }
            proc_macro2::TokenTree::Ident(idt) => {
                if &idt == ident {
                    // retrain the orignal span
                    let lit_value = syn::LitInt::new(&i.to_string(), idt.span());
                    new_code_block.extend(lit_value.to_token_stream());
                } else {
                    new_code_block.extend(idt.to_token_stream());
                }
            }
            proc_macro2::TokenTree::Punct(pct) => new_code_block.extend(pct.to_token_stream()),
            proc_macro2::TokenTree::Literal(lit) => new_code_block.extend(lit.to_token_stream()),
        }
    }
    new_code_block
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);
    // eprintln!("INPUT: {:#?}", input);

    let start = input.start.base10_parse::<usize>().unwrap();
    let end = input.end.base10_parse::<usize>().unwrap();

    let mut code_blocks = Vec::new();
    for i in start..end {
        let code_block =
            update_codeblock(input.code_block.clone().to_token_stream(), &input.ident, i);
        code_blocks.push(code_block);
    }
    // eprintln!("OUTPUT: {:#?}", code_blocks);

    let expanded = quote! {
        #(#code_blocks)*
    };
    expanded.into()
}
