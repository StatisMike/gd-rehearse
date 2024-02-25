/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::cases::{Case, CaseOutcome};
use crate::registry::bench::BenchResult;
use crate::runner::extract_file_subtitle;

use super::config::{RunnerConfig, RunnerInfo};
use super::is_headless_run;

use godot::log::godot_print;

pub(crate) struct MessageWriter {
    to_godot: bool,
    quiet: bool,
}

impl MessageWriter {
    pub fn new(quiet: bool) -> Self {
        let to_godot = !is_headless_run();
        Self { to_godot, quiet }
    }

    pub fn to_godot(&self) -> bool {
        self.to_godot
    }

    pub fn println(&self, message: &str) {
        if self.quiet {
            return;
        }
        if self.to_godot() {
            godot_print!("{}", message);
        } else {
            println!("{}", message)
        }
    }

    pub fn print_begin(&self) {
        self.println(&format!(
            "{hf}\n{hp}{h:^40}{hp}\n{hf}",
            hf = "-".repeat(80),
            hp = "-".repeat(20),
            h = "Running gd-rehearse"
        ));
    }

    pub fn print_success(&self) {
        self.println(&format!(
            "\n{hf}\n{hp}{h:^38}{hp}\n{hf}",
            hf = "= ".repeat(40),
            hp = " =".repeat(10),
            h = "! SUCCESS !"
        ));
    }

    pub fn print_failure(&self) {
        self.println(&format!(
            "\n{hf}\n{hp}{h:^38}{hp}\n{hf}",
            hf = "! ".repeat(40),
            hp = " !".repeat(10),
            h = "FAILURE"
        ));
    }

    pub fn print_horizontal_separator(&self) {
        self.println(&"-".repeat(80).to_string());
    }

    pub fn print_summary_info(&self, config: &RunnerConfig) {
        let RunnerInfo {
            mode,
            rust_build,
            godot_build,
            additional_message,
        } = RunnerInfo::gather(config);

        self.println(&format!(
            "{:^80}",
            format!(
                "Began run in {mode} mode in scene: {scene_path}",
                scene_path = &config.scene_path()
            )
        ));

        self.println(&format!(
            "{:^80}",
            format!("Rust build: {rust_build}; Godot build: {godot_build}")
        ));

        if !additional_message.is_empty() {
            self.println(&format!("{:^80}", additional_message.join(" & ")));
        }

        if !config.filters().is_empty() {
            self.println(&format!(
                "   Using filters:\n   * {}\n",
                config.filters().join("\n   * ")
            ));
        }

        self.println("");
    }

    pub fn print_test_pre(&self, test: impl Case, last_file: &mut Option<String>) {
        if self.quiet {
            return;
        }
        self.print_file_header(test.get_case_file(), last_file);
        // If printing to Godot console, the result will be printed as a whole string. That's because every `godot_print!` prints
        // the whole line only and there is no alternative for appending to the godot console output.
        if self.to_godot {
            return;
        }
        print!("   -- {} ... ", test.get_case_name());
    }

    fn print_file_header(&self, file: &str, last_file: &mut Option<String>) {
        // Check if we need to open a new category for a file.
        let is_new_file = last_file
            .as_ref()
            .map_or(true, |last_file| last_file != file);

        if !is_new_file {
            return;
        }

        // For the first case to omit `\n`.
        if last_file.is_none() {
            self.println(&format!("   {}:", extract_file_subtitle(file)));
        } else {
            self.println(&format!("\n   {}:", extract_file_subtitle(file)));
        }
        // State update for file-category-print
        *last_file = Some(file.to_owned());
    }

    pub fn print_test_post(&self, test_case: &str, outcome: CaseOutcome) {
        if self.quiet {
            return;
        }
        match (self.to_godot, &outcome) {
            // For printing from godot, always print the whole line, as `print_test_pre` didn't print anything for the case.
            (true, _) => {
                self.println(&format!("   -- {test_case} ... {outcome}"));
            }
            // If to stdout, print the whole line only in case of error, as if test failed, something was printed (e.g. an assertion), so we can
            // print the entire line again;
            (false, CaseOutcome::Failed) => {
                self.println(&format!("\n   -- {test_case} ... {outcome}"));
            }
            // Otherwise just outcome on same line.
            (false, _) => {
                println!("{outcome}");
            }
        }
    }

    pub fn print_bench_pre(&self, benchmark: &impl Case, last_file: &mut Option<String>) {
        if self.quiet {
            return;
        }
        self.print_file_header(benchmark.get_case_file(), last_file);

        // If printing to Godot console, the result will be printed as a whole string. That's because every `godot_print!` prints
        // the whole line only and there is no alternative for appending to the godot console output.
        if self.to_godot {
            return;
        }
        let benchmark_name = if benchmark.get_case_name().len() > 26 {
            &benchmark.get_case_name()[..26]
        } else {
            benchmark.get_case_name()
        };

        print!("   -- {benchmark_name:<26} ...");
    }

    pub fn print_bench_post(&self, benchmark: &str, result: BenchResult) {
        if self.quiet {
            return;
        }
        let adjusted_name = if benchmark.len() > 26 {
            &benchmark[..26]
        } else {
            benchmark
        };

        let outcome = match &result.outcome {
            CaseOutcome::Passed => {
                let mut outcome = String::new();
                for stat in result.stats.iter() {
                    outcome.push_str(&format!(" {:>10.3}μs", stat.as_nanos() as f64 / 1000.0));
                }
                outcome
            }
            _ => format!("    {}", result.outcome),
        };

        match (self.to_godot, &result.outcome) {
            // For printing from godot, always print the whole line, as `print_test_pre` didn't print anything for the case.
            (true, _) => {
                self.println(&format!("   -- {adjusted_name:<26} ...{outcome}"));
            }
            // If to stdout, print the whole line only in case of error, as if test failed, something was printed (e.g. an assertion), so we can
            // print the entire line again;
            (false, CaseOutcome::Failed) => {
                self.println(&format!("\n   -- {adjusted_name:<26} ...{outcome}"));
            }
            // Otherwise just outcome on same line.
            (false, _) => {
                println!("{outcome\n}");
            }
        }
    }
}
