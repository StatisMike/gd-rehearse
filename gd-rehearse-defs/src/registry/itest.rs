/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::cases::rust_test_case::RustTestCase;
use crate::runner::config::RunnerConfig;

use super::CaseFilterer;

godot::sys::plugin_registry!(pub GD_REHEARSE_RUST_TEST_CASES: RustTestCase);

#[doc(hidden)]
pub(crate) struct GdRustItests {
    tests: Vec<RustTestCase>,
    files_count: usize,
    is_focus_run: bool,
    is_path_run: bool,
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

    pub(crate) fn init(config: &RunnerConfig) -> Self {
        let mut instance = Self {
            tests: Vec::new(),
            files_count: 0,
            is_focus_run: false,
            is_path_run: false,
        };

        instance.collect_rust_tests();
        instance.is_path_run = instance.is_any_path_eq(config.scene_path()) || instance.is_path_run;

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

impl CaseFilterer<RustTestCase> for GdRustItests {
    fn is_path_run(&self) -> bool {
        self.is_path_run
    }
    fn set_path_run(&mut self, is_path_run: bool) {
        self.is_path_run = is_path_run
    }
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
