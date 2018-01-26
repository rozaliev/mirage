#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::*;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn async(_attribute: TokenStream, function: TokenStream) -> TokenStream {
    let mut i: Item = syn::parse(function).unwrap();
    {
        let item_fn = match i {
            Item::Fn(ref mut item) => item,
            _ => panic!("async attr can only be used on functions"),
        };

        let n_block = {
            let block = &item_fn.block;
            let t_block = quote! {
                {
                    macro_rules! i {
                        ($($b:tt)*) => {
                             $($b)*
                        };
                    }
                    i! {
                        ::mirage_async::AsAsync(unsafe { static move || {
                            if false { yield };
                            #block
                        }})
                    }
                }
            };
            Box::new(syn::parse(t_block.into()).unwrap())
        };

        item_fn.block = n_block;
    }

    (quote! {
        #i
    }).into()
}
