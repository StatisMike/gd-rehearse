/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::any::Any;
use std::error::Error;
use std::fmt::Display;
use std::panic::UnwindSafe;

pub(crate) type UnwindResult<T> = Result<Result<T, Box<dyn Any + Send>>, Box<dyn Any + Send>>;

pub(crate) fn unwind_result<T>(res: UnwindResult<T>) -> Result<T, UnwindError> {
    match res {
        Ok(inner_res) => match inner_res {
            Ok(ok) => Ok(ok),
            Err(err) => Err(unpack_err(err)),
        },
        Err(err) => Err(unpack_err(err)),
    }
}

fn unpack_err(err: Box<dyn Any + Send>) -> UnwindError {
    match err.downcast_ref::<&str>() {
        Some(str) => UnwindError {
            message: str.to_string(),
        },
        None => match err.downcast_ref::<String>() {
            Some(string) => UnwindError {
                message: string.to_owned(),
            },
            None => UnwindError {
                message: "cannot retrieve panic message".to_owned(),
            },
        },
    }
}

#[derive(Debug)]
pub(crate) struct UnwindError {
    message: String,
}

impl Display for UnwindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for UnwindError {}

pub(crate) fn handle_panic<C>(code: C) -> Result<(), UnwindError>
where
    C: FnOnce() + UnwindSafe,
{
    let result: UnwindResult<()> = std::panic::catch_unwind(move || {
        (code)();
        Ok(())
    });
    unwind_result(result)
}
