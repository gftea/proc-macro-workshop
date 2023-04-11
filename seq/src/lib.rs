use std::ops::Range;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseBuffer},
    parse_macro_input,
};

#[derive(Debug)]
#[allow(dead_code)]
struct Seq {
    ident: syn::Ident,
    start: usize,
    end: usize,
    code_block: proc_macro2::TokenStream,
}

fn interpolate_codeblock(
    code_block: ParseBuffer,
    ident: &syn::Ident,
    i: usize,
    tokens: &mut proc_macro2::TokenStream,
    range: Range<usize>,
    repeat: &mut bool,
) -> syn::Result<()> {
    while !code_block.is_empty() {
        if code_block.peek(syn::Ident) {
            let idt: syn::Ident = code_block.parse()?;

            if code_block.peek(syn::Token![~]) && code_block.peek2(syn::Ident) {
                code_block.parse::<syn::Token![~]>()?;
                let idt2: syn::Ident = code_block.parse()?;
                if &idt2 == ident {
                    *repeat = true;
                    // replace with new ident
                    let new_idt = syn::Ident::new(&format!("{}{}", idt, i), idt.span());
                    new_idt.to_tokens(tokens);
                } else {
                    eprintln!("error ident sequence: {}~{}", idt, idt2);
                    // unrecognized ident sequence, just remove the '~' and paste the two idents
                    let new_idt = syn::Ident::new(&format!("{}{}", idt, idt2), idt.span());
                    new_idt.to_tokens(tokens);
                }
            } else {
                if &idt == ident {
                    *repeat = true;
                    // replace with value of i
                    let lit_value = syn::LitInt::new(&format!("{}", i), idt.span());
                    lit_value.to_tokens(tokens);
                } else {
                    idt.to_tokens(tokens);
                }
            }
        } else if code_block.peek(syn::Token![#]) && code_block.peek2(syn::token::Paren) {
            // parse and discard the '#'
            code_block.parse::<syn::Token![#]>()?;

            let ahead = code_block.fork();
            let inner;
            parenthesized!(inner in ahead);

            // parse and discard parenthesized group
            code_block.parse::<proc_macro2::Group>()?;

            let mut new_tokens = proc_macro2::TokenStream::new();
            for i in range.clone() {
                // always repeat sections in parenthesized groups '#( ... )*'
                let mut repeat = true;
                interpolate_codeblock(
                    inner.fork(),
                    ident,
                    i,
                    &mut new_tokens,
                    range.clone(),
                    &mut repeat,
                )?;
                // if !repeat {
                //     break;
                // }
            }
            // parse and discard '*'
            code_block.parse::<syn::Token![*]>()?;
            new_tokens.to_tokens(tokens);
        } else {
            // if it is a Group, recurse
            if code_block.peek(syn::token::Brace)
                || code_block.peek(syn::token::Bracket)
                || code_block.peek(syn::token::Paren)
            {
                let ahead = code_block.fork();
                let inner;
                if code_block.peek(syn::token::Brace) {
                    braced!(inner in ahead);
                } else if code_block.peek(syn::token::Bracket) {
                    bracketed!(inner in ahead);
                } else if code_block.peek(syn::token::Paren) {
                    parenthesized!(inner in ahead);
                } else {
                    unreachable!();
                }

                let mut new_tokens = proc_macro2::TokenStream::new();
                interpolate_codeblock(inner, ident, i, &mut new_tokens, range.clone(), repeat)?;
                // consume the existing group in parse buffer,
                // and construct new group to token stream
                let g: proc_macro2::Group = code_block.parse()?;
                let mut new_group = proc_macro2::Group::new(g.delimiter(), new_tokens);
                new_group.set_span(g.span());
                new_group.to_tokens(tokens);
            } else {
                let tt: proc_macro2::TokenTree = code_block.parse()?;
                tt.to_tokens(tokens);
            }
        }
    }
    Ok(())
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start: usize = input.parse::<syn::LitInt>()?.base10_parse()?;
        input.parse::<syn::Token![..]>()?;
        let inclusive = if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            true
        } else {
            false
        };
        let end: usize = input.parse::<syn::LitInt>()?.base10_parse()?;
        let range = if inclusive {
            start..end + 1
        } else {
            start..end
        };

        let inner;
        braced!(inner in input);

        // let code_block = inner.parse::<proc_macro2::TokenStream>()?;
        let mut code_block = proc_macro2::TokenStream::new();
        for i in range.clone() {
            let mut repeat = false;
            let mut tks = proc_macro2::TokenStream::new();
            interpolate_codeblock(
                inner.fork(),
                &ident,
                i,
                &mut tks,
                range.clone(),
                &mut repeat,
            )?;

            // if the first scan of the code block  contain the repeated ident pattern
            // we need to scan the code block again to replace the ident with the value of i
            // otherwise, we only need to do once
            code_block.extend(tks);
            if !repeat {
                break;
            }
        }
        // eprintln!("code_block: {:#?}", code_block);
        // consume the buffer
        let _: proc_macro2::TokenStream = inner.parse()?;

        Ok(Seq {
            ident,
            start,
            end,
            code_block,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);

    // eprintln!("INPUT: {:#?}", input);

    let code_block = input.code_block;
    // eprintln!("OUTPUT: {:#?}", code_block);
    let expanded = quote! {
        #code_block
    };
    expanded.into()
}
