/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::{
    cases::{rust_test_case::RustTestCase, CaseOutcome},
    runner::panic::UnwindError,
};

use super::CaseFilterer;

godot::sys::plugin_registry!(pub GD_REHEARSE_RUST_TEST_CASES: RustTestCase);

#[doc(hidden)]
pub(crate) struct GdRustItests {
    tests: Vec<RustTestCase>,
    files_count: usize,
    is_focus_run: bool,
}

impl GdRustItests {
    pub fn get_test(&mut self) -> Option<RustTestCase> {
        self.tests.pop()
    }

    pub fn tests_count(&self) -> usize {
        self.tests.len()
    }

    pub fn files_count(&self) -> usize {
        self.files_count
    }

    pub(crate) fn init() -> Self {
        let mut instance = Self {
            tests: Vec::new(),
            files_count: 0,
            is_focus_run: false,
        };

        instance.collect_rust_tests();

        instance
    }

    pub fn get_post_init_summary(&self) -> String {
        format!(
            "   Found {} Rust tests in {} files",
            self.tests_count(),
            self.files_count()
        )
    }

    fn get_rust_case() -> Option<RustTestCase> {
        __godot_rust_plugin_GD_REHEARSE_RUST_TEST_CASES
            .lock()
            .expect("can't retrieve RustTestCase")
            .pop()
    }

    fn collect_rust_tests(&mut self) {
        while let Some(test) = Self::get_rust_case() {
            self.tests.push(test);
        }
    }

    pub fn finish_setup(&mut self) {
        self.sort_cases();
        self.files_count = self.get_files_count()
    }
}

pub(crate) struct TestResult {
    pub(crate) outcome: CaseOutcome,
    pub(crate) error: Option<UnwindError>,
}

impl TestResult {
    pub fn success() -> Self {
        Self {
            outcome: CaseOutcome::Passed,
            error: None,
        }
    }

    pub fn skipped() -> Self {
        Self {
            outcome: CaseOutcome::Skipped,
            error: None,
        }
    }

    pub fn failed(err: UnwindError) -> Self {
        Self {
            outcome: CaseOutcome::Failed,
            error: Some(err),
        }
    }
}

impl CaseFilterer<RustTestCase> for GdRustItests {
    fn is_focus_run(&self) -> bool {
        self.is_focus_run
    }
    fn set_focus_run(&mut self, is_focus_run: bool) {
        self.is_focus_run = is_focus_run
    }
    fn get_cases(&self) -> &Vec<RustTestCase> {
        &self.tests
    }
    fn get_cases_mut(&mut self) -> &mut Vec<RustTestCase> {
        &mut self.tests
    }
}
