use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse::Parse, parse_macro_input, parse_quote, Expr, Type};
// use std::marker::PhantomData;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item_struct = parse_macro_input!(input as syn::ItemStruct);
    let vis = item_struct.vis;
    let name = item_struct.ident;
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut offsets = Vec::new();
    let mut total_bits = quote!(0usize);
    for field in item_struct.fields {
        let ty = &field.ty;
        
        if let Type::Path(ty_path) = ty {
            let bit_width: Expr = parse_quote!(<#ty_path as Specifier>::BITS);            
            offsets.push(total_bits.clone());
            total_bits.extend(quote!(+ #bit_width));
        }
        field_names.push(field.ident.unwrap());
        field_types.push(field.ty);
    }
   
    let get_field_names = field_names.iter().map(|field_name| {
        format_ident!("get_{}", field_name)
    });
    let set_field_names = field_names.iter().map(|field_name| {
        format_ident!("set_{}", field_name)
    });

  
    quote!(
        // use bitfield::checks::*;

        const TOTAL_BITS_MOD8: usize = (#total_bits) % 8; 
        const ARRAY_SIZE: usize = (#total_bits) / 8; 
        require_multiple_of_eight!(TOTAL_BITS_MOD8);


        #[repr(C)]
        #vis struct #name {
            data: [u8; ARRAY_SIZE],
        }       
        impl #name {
            pub fn new() -> Self {                
                Self {
                    data: [0; ARRAY_SIZE],
                }
            }
            
            fn get_bit_seq(&self, start: usize, end: usize) -> Vec<bool> {
                let mut bits = Vec::new();
            
                for index in start..end {
                    let byte = self.data[index / 8];
                    let bit = byte >> (7 - index % 8) & 1;
                    bits.push(bit == 1);
                }
            
                bits
            }

            fn set_bit_seq(&mut self, value: u64, start: usize, end: usize) {
                let mut value = value;
                for index in start..end {
                    let byte = self.data[index / 8];
                    let bit = (value & 1) as u8;
                    self.data[index / 8] = byte | (bit <<(7 - index % 8));
                    eprintln!("{} {:08b} {:08b}", index, byte, self.data[index / 8]);
                    value >>= 1;
                }
            }
            
            #(pub fn #get_field_names(&self) -> u64 {
                let bit_seq = self.get_bit_seq(#offsets, #offsets + <#field_types as Specifier>::BITS);

                let mut value = 0u64;
                for (i, bit) in bit_seq.iter().enumerate() {
                    value |= (*bit as u64) << i;    
                }
                value
            })*

            #(pub fn #set_field_names(&mut self, value: u64) {
                self.set_bit_seq(value, #offsets, #offsets + <#field_types as Specifier>::BITS);
            })*

        }
    )
    .into()
}

/// The `bitspec` macro is used to define a bit specification.
/// bitspec!(Ident, Literal);
/// # Example
/// bitspec!(B1, 1);
/// bitspec!(B2, 2);
#[proc_macro]
pub fn bitspec(input: TokenStream) -> TokenStream {
    let spec = parse_macro_input!(input as BitFieldSpec);
    let name = spec.ident;
    let width = spec.width;
    quote!(
        // uninhabited type
        pub enum #name {}

        impl Specifier for #name {
            const BITS: usize = #width;
        }
    )
    .into()
}

struct BitFieldSpec {
    ident: syn::Ident,
    width: syn::LitInt,
}

impl Parse for BitFieldSpec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![,]>()?;
        let width = input.parse::<syn::LitInt>()?;

        Ok(Self { ident, width })
    }
}


