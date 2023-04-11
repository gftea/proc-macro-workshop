use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
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
) -> syn::Result<()> {
    while !code_block.is_empty() {
        if code_block.peek(syn::Ident) {
            let idt: syn::Ident = code_block.parse()?;

            if code_block.peek(syn::Token![~]) && code_block.peek2(syn::Ident) {
                code_block.parse::<syn::Token![~]>()?;
                let idt2: syn::Ident = code_block.parse()?;
                if &idt2 == ident {
                    // construct new ident with i
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
                    // replace with value of i
                    let lit_value = syn::LitInt::new(&format!("{}", i), idt.span());
                    lit_value.to_tokens(tokens);
                } else {
                    idt.to_tokens(tokens);
                }
            }
        } else {
            // if it is a Group, recurse
            if code_block.peek(syn::token::Brace) {
                let ahead = code_block.fork();
                let inner;
                braced!(inner in ahead);

                let mut new_tokens = proc_macro2::TokenStream::new();
                interpolate_codeblock(inner, ident, i, &mut new_tokens)?;
                // reconstruct the group                
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
        let end: usize = input.parse::<syn::LitInt>()?.base10_parse()?;

        let inner;
        braced!(inner in input);
        
        
        // let code_block = inner.parse::<proc_macro2::TokenStream>()?;
        let mut code_block = proc_macro2::TokenStream::new();
        
        let i = 1;
        interpolate_codeblock(inner, &ident, i, &mut code_block)?;
        eprintln!("code_block: {:#?}", code_block);
        

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

    eprintln!("INPUT: {:#?}", input);
 
    let code_block = input.code_block;
    eprintln!("OUTPUT: {:#?}", code_block);
    let expanded = quote! {
        #code_block
    };
    expanded.into()
}
