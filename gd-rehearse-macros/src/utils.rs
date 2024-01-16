/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use venial::Declaration;

pub fn bail_fn<R, T>(msg: impl AsRef<str>, tokens: T) -> Result<R, venial::Error>
where
    T: quote::spanned::Spanned,
{
    // TODO: using T: Spanned often only highlights the first tokens of the symbol, e.g. #[attr] in a function.
    // Could use function.name; possibly our own trait to get a more meaningful span... or change upstream in venial.

    Err(error_fn(msg, tokens))
}

pub fn error_fn<T>(msg: impl AsRef<str>, tokens: T) -> venial::Error
where
    T: quote::spanned::Spanned,
{
    venial::Error::new_at_span(tokens.__span(), msg.as_ref())
}

macro_rules! bail {
    ($tokens:expr, $format_string:literal $($rest:tt)*) => {
        $crate::utils::bail_fn(format!($format_string $($rest)*), $tokens)
    }
}

pub(crate) use bail;

pub(crate) fn translate_meta<F>(
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
