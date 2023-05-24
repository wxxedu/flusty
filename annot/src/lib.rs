use proc_macro2::TokenStream;
use syn::{parse_macro_input, Item, ItemEnum, ItemFn, ItemStruct};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn flusty(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as Item);
    handle_item(&item).into()
}

fn handle_item(item: &Item) -> TokenStream {
    match item {
        Item::Fn(f) => handle_item_fn(f),
        Item::Struct(s) => handle_item_struct(s),
        Item::Enum(e) => handle_item_enum(e),
        _ => panic!("flusty can only be used on functions"),
    }
}

fn handle_item_fn(f: &ItemFn) -> TokenStream {
    let sig = &f.sig;
    let body = &f.block;
    quote::quote! {
        #[no_mangle]
        pub extern "C" #sig {
            #body
        }
    }
}

fn handle_item_struct(s: &ItemStruct) -> TokenStream {
    quote::quote! {
        #[repr(C)]
        #s
    }
}

fn handle_item_enum(e: &ItemEnum) -> TokenStream {
    quote::quote! {
        #[repr(C)]
        #e
    }
}
