extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Stmt};
use rand::{Rng, thread_rng};

#[proc_macro_attribute]
pub fn obfuscate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let mut rng = thread_rng();

    let mut new_stmts = Vec::new();

    for stmt in input.block.stmts {
        // Inject junk code before each statement with 70% probability
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
                // This branch is never taken but adds noise to static analysis
                let _z = #val ^ 0xDEADBEEF;
                std::hint::black_box(_z);
            }
        },
    }
}
