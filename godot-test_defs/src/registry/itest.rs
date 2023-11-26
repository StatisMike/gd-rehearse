/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::collections::HashSet;

use crate::cases::rust_test_case::RustTestCase;
use crate::cases::Case;
use crate::runner::config::RunnerConfig;

godot::sys::plugin_registry!(pub GODOT_TEST_RUST_TEST_CASES: RustTestCase);

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

    pub fn is_focus_run(&self) -> bool {
        self.is_focus_run
    }

    pub(crate) fn init(config: &RunnerConfig, is_focus_run: bool) -> Self {
        let mut instance = Self {
            tests: Vec::new(),
            files_count: 0,
            is_focus_run,
        };

        instance.collect_rust_tests(config);
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
        __godot_rust_plugin_GODOT_TEST_RUST_TEST_CASES
            .lock()
            .expect("can't retrieve RustTestCase")
            .pop()
    }

    fn collect_rust_tests(&mut self, config: &RunnerConfig) {
        let mut all_files = HashSet::new();

        while let Some(test) = Self::get_rust_case() {
            // Collect only tests based on keyword. If keyword in runner is empty, all will pass this check
            if test.should_run_keyword(config.keyword(), config.ignore_keywords()) {
                if !self.is_focus_run && test.is_case_focus() && !config.disallow_focus() {
                    self.tests.clear();
                    all_files.clear();
                    self.is_focus_run = true;
                }

                if (!self.is_focus_run && test.should_run_filters(config.filters()))
                    || test.should_run_focus(config.disallow_focus())
                {
                    all_files.insert(test.file);
                    self.tests.push(test);
                }
            }
        }

        // Sort alphabetically for deterministic run order
        self.tests
            .sort_by_key(|test| format!("{}{}", test.file, test.name));

        self.files_count = all_files.len();
    }
}
