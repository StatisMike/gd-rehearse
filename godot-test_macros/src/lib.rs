/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use venial::Declaration;

mod bench;
mod itest;
pub(crate) mod parser;
mod utils;

/// Integration test between Godot and Rust.
/// 
/// Similar to `#[test]`, but converting the function into integration test between Godot and Rust.
/// 
/// It transforms and registers annotated function for further usage by [godot_test]
/// 
/// 
///
/// Transforms the `fn` into one returning `bool` (success of the test), which must be called explicitly.
#[proc_macro_attribute]
pub fn gditest(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gditest", meta, input, itest::attribute_gditest)
}

#[proc_macro_attribute]
pub fn gdbench(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gdbench", meta, input, bench::attribute_bench)
}

fn translate_meta<F>(
    self_name: &str,
    meta: TokenStream,
    input: TokenStream,
    transform: F,
) -> TokenStream
where
    F: FnOnce(Declaration) -> Result<TokenStream2, venial::Error>,
{
    let self_name = format_ident!("{}", self_name);
    let input2 = TokenStream2::from(input);
    let meta2 = TokenStream2::from(meta);

    // Hack because venial doesn't support direct meta parsing yet
    let input = quote! {
        #[#self_name(#meta2)]
        #input2
    };

    let result2 = venial::parse_declaration(input)
        .and_then(transform)
        .unwrap_or_else(|e| e.to_compile_error());

    TokenStream::from(result2)
}
