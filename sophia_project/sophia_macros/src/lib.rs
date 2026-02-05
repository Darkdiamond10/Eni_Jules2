extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, Stmt};
use rand::{Rng, thread_rng};

#[proc_macro_attribute]
pub fn obfuscate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let mut rng = thread_rng();

    let mut new_stmts = Vec::new();

    for stmt in input.block.stmts {
        if rng.gen_bool(0.7) {
            let junk = generate_junk_stmt(&mut rng);
            new_stmts.push(junk);
        }
        new_stmts.push(stmt);
    }

    input.block.stmts = new_stmts;

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn encrypt_string(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();
    let bytes = input_str.as_bytes();
    let len = bytes.len();

    let mut rng = thread_rng();
    let key: u8 = rng.gen_range(1..255);

    let encrypted_bytes: Vec<u8> = bytes.iter().map(|b| b ^ key).collect();

    let expanded = quote! {
        {
            let mut data = [#(#encrypted_bytes),*];
            for i in 0..#len {
                data[i] ^= #key;
            }
            String::from_utf8_lossy(&data).to_string()
        }
    };

    TokenStream::from(expanded)
}

fn generate_junk_stmt(rng: &mut impl Rng) -> Stmt {
    let val: u64 = rng.gen();
    let choice = rng.gen_range(0..3);

    match choice {
        0 => syn::parse_quote! {
            if false {
                let _x = #val + 1337;
                std::hint::black_box(_x);
            }
        },
        1 => syn::parse_quote! {
            {
                let mut _y = #val;
                _y = _y.wrapping_mul(3);
                _y = _y.wrapping_sub(1);
                std::hint::black_box(_y);
            }
        },
        _ => syn::parse_quote! {
            if (1 + 1) == 3 {
                let _z = #val ^ 0xDEADBEEF;
                std::hint::black_box(_z);
            }
        },
    }
}
