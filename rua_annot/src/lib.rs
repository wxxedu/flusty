//! # `rua-annot`
//!
//! This crate provides the annotations needed to generate binding between Rust
//! and another language. (Currently, Dart).
//!
//! Currently, it provides the attribute macro `#[flusty]` which can be used on
//! functions, structs, and enums (in the working). The macro will also make
//! the code in Rust compile in the C ABI, making it possible in the FFI.
#![warn(clippy::all, missing_docs)]
use proc_macro2::TokenStream;
use syn::{parse_macro_input, Item, ItemEnum, ItemFn, ItemStruct};

extern crate proc_macro;

/// The attribute macro that makes the code in Rust compile in the C ABI.
/// - If applied to a function, say `fn foo() -> i32`, it will make the
///   function `#[no_mangle] pub extern "C" fn foo() -> i32`.
/// - If applied to a struct or enum, it will make the struct or enum
///   `#[repr(C)]`.
/// - If applied to anything else, it will panic.
#[proc_macro_attribute]
pub fn rua(
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
