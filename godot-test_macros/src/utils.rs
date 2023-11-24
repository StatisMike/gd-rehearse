/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

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

