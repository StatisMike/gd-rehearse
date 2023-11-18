use std::collections::HashSet;

use crate::classes::RustTestCase;

static RUST_TEST_CASES: std::sync::Mutex<Vec<RustTestCase>> = std::sync::Mutex::new(Vec::new());

pub fn register_rust_case(case: RustTestCase) {
    RUST_TEST_CASES
        .lock()
        .expect("can't add RustTestCase")
        .push(case);
}

pub struct GdRustItests {
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

    pub fn init(filters: &[String]) -> Self {
        let mut instance = Self {
            tests: Vec::new(),
            files_count: 0,
            is_focus_run: false,
        };

        instance.collect_rust_tests(filters);
        instance
    }

    fn passes_filter(filters: &[String], test_name: &str) -> bool {
        filters.is_empty() || filters.iter().any(|x| test_name.contains(x))
    }

    fn get_rust_case() -> Option<RustTestCase> {
        RUST_TEST_CASES
            .lock()
            .expect("can't retrieve RustTestCase")
            .pop()
    }

    fn collect_rust_tests(&mut self, filters: &[String]) {
        let mut all_files = HashSet::new();

        while let Some(test) = Self::get_rust_case() {
            if !self.is_focus_run && test.focused {
                self.tests.clear();
                all_files.clear();
                self.is_focus_run = true;
            }

            // Only collect tests if normal mode, or focus mode and test is focused.
            if (!self.is_focus_run || test.focused) && Self::passes_filter(filters, test.name) {
                all_files.insert(test.file);
                self.tests.push(test);
            }
        }

        // Sort alphabetically for deterministic run order
        self.tests.sort_by_key(|test| test.file);

        self.files_count = all_files.len();
    }
}

pub struct GodotTests {}
