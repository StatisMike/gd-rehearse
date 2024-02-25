/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use godot::engine::Node;
use godot::obj::Gd;

use super::{Case, CaseContext};

/// Rust test case.
///
/// Created by using `#[gditest]` macro and registered to run by test runner.
#[derive(Copy, Clone)]
pub struct RustTestCase {
    pub name: &'static str,
    pub file: &'static str,
    pub skipped: bool,
    /// If one or more tests are focused, only they will be executed. Helpful for debugging and working on specific features.
    pub focused: bool,
    /// Used in conjuction with set
    pub keyword: Option<&'static str>,
    pub scene_path: Option<&'static str>,
    #[allow(dead_code)]
    pub line: u32,
    pub function: fn(&TestContext),
}

impl Case for RustTestCase {
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
    fn get_case_scene_path(&self) -> &Option<&str> {
        &self.scene_path
    }
    fn get_case_line(&self) -> u32 {
        self.line
    }
}

/// Optional test context for `#[gditest]`.
///
/// Allows accessing [GdTestRunner](crate::runner::GdTestRunner) scene tree during tests.
pub struct TestContext {
    pub(crate) scene_tree: Gd<Node>,
}

impl TestContext {
    pub(crate) fn new(scene_tree: Gd<Node>) -> Self {
        Self { scene_tree }
    }
}

impl CaseContext for TestContext {
    fn scene_tree(&self) -> &Gd<Node> {
        &self.scene_tree
    }
}
