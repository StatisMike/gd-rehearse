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

#[doc(hidden)]
/// Signal to the compiler that a value is used (to avoid optimization).
pub fn bench_used<T: Sized>(value: T) {
    std::hint::black_box(value);
}
