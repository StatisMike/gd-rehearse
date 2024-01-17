/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

pub mod gd_test_case;
pub mod rust_bench;
pub mod rust_test_case;

use std::cmp::Ordering;

use godot::engine::{Engine, Node};
use godot::obj::Gd;

/// Optional test context for `#[gditest]` and `#[gdbench]` annotated functions.
///
/// Currently it allows only to access [GdTestRunner](crate::runner::GdTestRunner) scene tree during tests and benchmarking.
pub struct CaseContext {
    pub(crate) scene_tree: Gd<Node>,
}

impl CaseContext {
    pub fn scene_tree(&self) -> &Gd<Node> {
        &self.scene_tree
    }
}

/// Case outcome.
#[must_use]
pub(crate) enum CaseOutcome {
    Passed,
    Failed,
    Skipped,
}

impl CaseOutcome {
    pub fn from_bool(success: bool) -> Self {
        if success {
            Self::Passed
        } else {
            Self::Failed
        }
    }
}

impl std::fmt::Display for CaseOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Do not use print_rich() from Godot, because it's very slow and significantly delays test execution.
        let outcome = match self {
            CaseOutcome::Passed => "ok!",
            CaseOutcome::Failed => "FAILED",
            CaseOutcome::Skipped => "~skipped~",
        };
        f.write_str(outcome)
    }
}

/// Disable printing errors from Godot. Ideally we should catch and handle errors, ensuring they happen when
/// expected. But that isn't possible, so for now we can just disable printing the error to avoid spamming
/// the terminal when tests should error.
pub fn suppress_godot_print(mut f: impl FnMut()) {
    Engine::singleton().set_print_error_messages(false);
    f();
    Engine::singleton().set_print_error_messages(true);
}

pub(crate) trait Case {
    fn is_case_focus(&self) -> bool;
    fn is_case_skip(&self) -> bool;
    fn get_case_keyword(&self) -> &Option<&str>;
    fn get_case_name(&self) -> &str;
    fn get_case_line(&self) -> u32;
    fn get_case_file(&self) -> &str;
    fn get_case_scene_path(&self) -> &Option<&str>;

    fn order(first: &Self, other: &Self) -> Ordering {
        other.get_order_string().cmp(&first.get_order_string())
    }

    fn get_order_string(&self) -> String {
        format!("{}{:06}", self.get_case_file(), self.get_case_line())
    }

    fn should_run_focus(&self, is_focus_run: bool) -> bool {
        !is_focus_run || self.is_case_focus()
    }

    fn should_run_skip(&self, disallow_skip: bool) -> bool {
        !self.is_case_skip() || disallow_skip
    }

    fn should_run_keyword(&self, keyword: &str, ignore_keywords: bool) -> bool {
        if ignore_keywords {
            return true;
        };
        if let Some(case_keyword) = self.get_case_keyword() {
            return *case_keyword == keyword;
        } else if keyword.is_empty() {
            return true;
        }
        false
    }

    fn should_run_filters(&self, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        };
        filters
            .iter()
            .any(|filter| self.get_case_name().contains(filter))
    }

    fn should_run_scene_path(&self, scene_path: &str, is_path_run: bool) -> bool {
        if !is_path_run && self.get_case_scene_path().is_none(){
            return true;
        }
        self.scene_path_eq(scene_path)
    }

    fn scene_path_eq(&self, scene_path: &str) -> bool {
        if let Some(path) = self.get_case_scene_path() {
            return *path == scene_path;
        }
        false
    }
}
