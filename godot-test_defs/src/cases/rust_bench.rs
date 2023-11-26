/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use super::{Case, CaseContext};

/// Rust benchmark.
///
/// Created by using `#[gdbench]` macro and registered to run by test runner.
#[derive(Copy, Clone)]
pub struct RustBenchmark {
    pub name: &'static str,
    pub file: &'static str,
    pub skipped: bool,
    pub focused: bool,
    pub keyword: Option<&'static str>,
    #[allow(dead_code)]
    pub line: u32,
    pub function: fn(&CaseContext),
    pub repetitions: usize,
}

impl Case for RustBenchmark {
    fn get_case_name(&self) -> &str {
        self.name
    }
    fn get_case_file(&self) -> &str {
        self.file
    }
    fn is_case_focus(&self) -> bool {
        self.focused
    }
    fn is_case_skip(&self) -> bool {
        self.skipped
    }
    fn get_case_keyword(&self) -> &Option<&str> {
        &self.keyword
    }
}

/// Signal to the compiler that a value is used (to avoid optimization).
pub fn bench_used<T: Sized>(value: T) {
    // The following check would be used to prevent `()` arguments, ensuring that a value from the bench is actually going into the blackbox.
    // However, we run into this issue, despite no array being used: https://github.com/rust-lang/rust/issues/43408.
    //   error[E0401]: can't use generic parameters from outer function
    // sys::static_assert!(std::mem::size_of::<T>() != 0, "returned unit value in benchmark; make sure to use a real value");

    std::hint::black_box(value);
}
