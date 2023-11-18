use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use venial::Declaration;

mod bench;
mod itest;
mod utils;

/// Similar to `#[test]`, but runs an integration test with Godot.
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
