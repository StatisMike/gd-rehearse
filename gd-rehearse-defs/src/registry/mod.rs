/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::collections::HashSet;

use crate::{cases::Case, runner::config::RunnerConfig};

pub mod bench;
pub mod itest;

pub(crate) trait CaseFilterer<T>
where
    T: Case,
{
    fn is_path_run(&self) -> bool;
    fn set_path_run(&mut self, is_path_run: bool);
    fn is_focus_run(&self) -> bool;
    fn set_focus_run(&mut self, is_focus_run: bool);
    fn get_cases(&self) -> &Vec<T>;
    fn get_cases_mut(&mut self) -> &mut Vec<T>;

    // Filter on path and keyword
    fn filter_path_keyword(&mut self, config: &RunnerConfig) {
        let is_path_run = self.is_path_run();
        // Retain only the ones with the specified path and keyword
        self.get_cases_mut().retain(|t| {
            t.should_run_scene_path(config.scene_path(), is_path_run)
                && t.should_run_keyword(config.keyword(), config.ignore_keywords())
        });
    }

    // Check for focus run
    fn check_focus_run(&mut self, config: &RunnerConfig) -> bool {
        !config.disallow_focus()
            && (self.is_focus_run() || self.get_cases().iter().any(|t| t.is_case_focus()))
    }

    // Filter on focus and filters
    fn filter_focus_filters(&mut self, config: &RunnerConfig) {
        let is_focus_run = self.check_focus_run(config) || self.is_focus_run();
        self.set_focus_run(is_focus_run);
        self.get_cases_mut()
            .retain(|c| c.should_run_focus(is_focus_run) && c.should_run_filters(config.filters()))
    }

    // Sort in deterministic order
    fn sort_cases(&mut self) {
        self.get_cases_mut().sort_by(|a, b| Case::order(a, b))
    }

    // Get files
    fn get_files_count(&self) -> usize {
        let mut set = HashSet::new();
        for case in self.get_cases().iter() {
            set.insert(case.get_case_file().to_owned());
        }
        set.len()
    }

    fn is_any_path_eq(&self, path: &str) -> bool {
        self.get_cases().iter().any(|c| c.scene_path_eq(path))
    }
}
