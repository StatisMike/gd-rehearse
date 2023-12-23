/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use godot::prelude::{godot_api, Base, GString, GodotClass, INode, Node, PackedStringArray};

use crate::cases::rust_bench::RustBenchmark;
use crate::cases::rust_test_case::RustTestCase;
use crate::cases::{Case, CaseContext, CaseOutcome};

use crate::registry::bench::{BenchResult, GdBenchmarks};
use crate::registry::itest::GdRustItests;

use super::config::RunnerConfig;
use super::extract_file_subtitle;
use super::print::MessageWriter;

use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct RunnerSummary {
    total: i64,
    passed: i64,
    skipped: i64,
}

impl RunnerSummary {
    pub fn inc_total(&mut self) {
        self.total += 1;
    }

    pub fn inc_passed(&mut self) {
        self.passed += 1;
    }

    pub fn inc_skipped(&mut self) {
        self.skipped += 1;
    }

    fn update_stats(&mut self, test: &impl Case, outcome: &CaseOutcome, failed: &mut Vec<String>) {
        self.inc_total();
        match outcome {
            CaseOutcome::Passed => self.inc_passed(),
            CaseOutcome::Failed => failed.push(format!(
                "{} > {}",
                extract_file_subtitle(test.get_case_file()),
                test.get_case_name()
            )),
            CaseOutcome::Skipped => self.inc_skipped(),
        }
    }

    fn conclude(&self, run_time: Duration, failed_list: &mut Vec<String>) -> bool {
        let Self {
            total,
            passed,
            skipped,
        } = *self;

        let writer = MessageWriter::new();

        // Consider 0 tests run as a failure too, because it's probably a problem with the run itself.
        let failed = total - passed - skipped;
        let all_passed = failed == 0 && total != 0;

        let outcome = CaseOutcome::from_bool(all_passed);

        let run_time = run_time.as_secs_f32();
        // let focused_run = self.focus_run;

        let extra = if skipped > 0 {
            format!(", {skipped} skipped")
        } else {
            "".to_string()
        };

        writer.println(&format!(
            "\nTest result: {outcome} {passed} passed; {failed} failed{extra}."
        ));
        writer.println(&format!("  Time: {run_time:.2}s."));

        if !all_passed {
            writer.println("\n  Failed tests:");
            let max = 10;
            for test in failed_list.iter().take(max) {
                writer.println(&format!("  * {test}"));
            }

            if failed_list.len() > max {
                writer.println(&format!("  * ... and {} more.", failed_list.len() - max));
            }

            failed_list.clear();

            writer.println("\n");
        }
        all_passed
    }
}

/// Tests and benchmark runner for custom Godot classes created using the [gdext](godot) crate.
///
/// Runs functions annotated with `#[gditest]` and `#[gdbench]` macros, facilitating the testing and benchmarking of methods and functions that involve calls between Rust and Godot. To utilize it, create a scene in the Godot project associated with your `gdext`-based GDExtension and run the scene either from the command line or directly from the Godot editor.
///
/// ## Godot Properties
///
/// `GdTestRunner` exposes some settable Godot-exported properties that customize its behavior:
///
/// - `run_tests`: If set, functions annotated with `#[gditest]` will be executed. Defaults to `true`.
/// - `run_benchmarks`: If set, functions annotated with `#[gdbench]` will be executed. Defaults to `true`. If `run_tests` is also `true`, benchmarks
///    will only be executed if all tests pass successfully.
/// - `test_keyword`: If set, only tests and benchmarks with the same `keyword` specified will be executed. Defaults to an empty string, meaning
///    that only tests and benchmarks without a `keyword` set will be executed. It takes precedence over `focus` and `filters`—they will be
///    assessed, but only in the context of this `keyword`.
/// - `ignore_keywords`: If set, all tests and benchmarks will be executed regardless of their set `keyword`.
/// - `disallow_focus`: If set, the `focus` attribute of tests and benchmarks will be ignored.
/// - `disallow_skip`: If set, the `skip` attribute of tests and benchmarks will be ignored.
/// - `test_filters`: An array of strings tested against the names of tests and benchmarks. Those with names containing at least one of the specified
///    filters will be executed.
///
/// ## Command Line Arguments
///
/// `GdTestRunner` is also suitable for running from the command line, as part of the Godot binary execution in headless mode:
///
/// ```no_compile
/// godot_executable --path path/to/godot/project [scene path] --headless -- [optional arguments]
/// ```
/// - `scene_path`: An optional path to the scene with the `GdTestRunner` object. Not needed if the testing scene is the main scene of the project.
/// - `optional arguments`: Options such as:
///   - `--rust-test` or `--rust-benchmarks`: If at least one is selected, overwrites the analogous properties and runs only the specified element.
///   - `--disallow-focus` or `--allow-focus`: Overwrites the `disallow_focus` property.
///   - `--disallow-skip` or `--disallow-focus`: Overwrites the `disallow_skip` property.
///   - `--mute-keyword` or `--keyword=my_keyword`: Either mutes the `test_keyword` property or replaces it with the specified one.
///   - `--ignore-keywords`: Replaces the `ignore_keywords` property.
///   - `--mute-filters` or `--filters=[filter1,filter2]`: Either mutes the `test_filters` property or replaces it with the specified filters.
///
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GdTestRunner {
    #[export]
    run_tests: bool,
    #[export]
    run_benchmarks: bool,
    #[export]
    disallow_focus: bool,
    #[export]
    disallow_skip: bool,
    #[export]
    test_keyword: GString,
    #[export]
    ignore_keywords: bool,
    #[export]
    test_filters: PackedStringArray,
    tests_summary: RunnerSummary,
    benches_summary: RunnerSummary,
    config: RunnerConfig,
    failed_list: Vec<String>,
    began_run: bool,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for GdTestRunner {
    fn init(base: Base<Node>) -> Self {
        Self {
            disallow_focus: false,
            disallow_skip: false,
            test_filters: PackedStringArray::new(),
            test_keyword: GString::new(),
            ignore_keywords: false,
            run_benchmarks: true,
            run_tests: true,
            tests_summary: RunnerSummary::default(),
            benches_summary: RunnerSummary::default(),
            config: RunnerConfig::default(),
            failed_list: Vec::new(),
            began_run: false,
            base,
        }
    }
    // Needed for the physics to be initialized for the tests that needs them
    fn ready(&mut self) {
        let mut scene_tree = self.base.get_tree().unwrap();
        scene_tree.connect("physics_frame".into(), self.base.callable("test_run"));
    }
}

#[godot_api]
impl GdTestRunner {
    #[func]
    fn test_run(&mut self) {
        if self.began_run {
            return;
        }

        self.began_run = true;
        let start = Instant::now();
        let writer = MessageWriter::new();

        match RunnerConfig::new(
            self.disallow_focus,
            self.disallow_skip,
            self.run_tests,
            self.run_benchmarks,
            &self.test_keyword,
            self.ignore_keywords,
            &self.test_filters,
        ) {
            Ok(config) => self.config = config,
            Err(error) => {
                writer.println(&error.to_string());
                self.end(1);
                return;
            }
        }

        writer.print_begin();

        self.config.print_info();

        let mut rust_test_outcome = true;
        let mut rust_bench_outcome = true;

        let mut rust_tests_handler: Option<GdRustItests> = None;
        let mut rust_bench_handler: Option<GdBenchmarks> = None;

        let mut is_focus_run = false;

        // Gather tests and benches to run based on the config.
        if self.config.run_rust_tests() {
            let handler = GdRustItests::init(&self.config, is_focus_run);
            writer.println(&handler.get_post_init_summary());
            is_focus_run = handler.is_focus_run();
            rust_tests_handler = Some(handler);
        }

        if self.config.run_rust_benchmarks() {
            let handler = GdBenchmarks::init(&self.config, is_focus_run);
            writer.println(&handler.get_post_init_summary());
            rust_bench_handler = Some(handler);
        }

        // Run Rust Tests.
        if let Some(mut handler) = rust_tests_handler {
            writer.println("");
            writer.print_horizontal_separator();
            writer.println("   Running Rust tests");
            writer.print_horizontal_separator();

            let clock = Instant::now();
            self.run_rust_tests(&mut handler);
            let run_time = clock.elapsed();
            rust_test_outcome = self.tests_summary.conclude(run_time, &mut self.failed_list);
        }

        // Run Rust Benchmarks.
        if let (Some(mut handler), true) = (rust_bench_handler, rust_test_outcome) {
            writer.println("");
            writer.print_horizontal_separator();
            writer.println("   Running Rust benchmarks");
            writer.print_horizontal_separator();

            let clock = Instant::now();
            self.run_rust_benchmarks(&mut handler);
            let run_time = clock.elapsed();
            rust_bench_outcome = self
                .benches_summary
                .conclude(run_time, &mut self.failed_list);
        }

        let outcome = rust_test_outcome && rust_bench_outcome;
        let duration = start.elapsed();
        let mut duration_secs = duration.as_secs();
        let duration_mins = (duration_secs as f32 / 60.).floor() as u64;
        duration_secs -= duration_mins * 60;

        if outcome {
            writer.print_success()
        } else {
            writer.print_failure()
        }
        writer.println(&format!(
            "\n  Took total: {duration_mins}:{duration_secs:0>2}"
        ));

        self.end(!outcome as i32);
    }

    fn end(&mut self, exit_code: i32) {
        self.base.queue_free();
        self.base
            .get_tree()
            .unwrap()
            .quit_ex()
            .exit_code(exit_code)
            .done();
    }

    fn run_rust_tests(&mut self, handler: &mut GdRustItests) {
        let ctx = CaseContext {
            scene_tree: self.base.clone().upcast(),
        };

        let writer = MessageWriter::new();

        let mut last_file = None;
        while let Some(test) = handler.get_test() {
            writer.print_test_pre(test, &mut last_file);

            let outcome = self.run_rust_test(&test, &ctx);
            self.tests_summary
                .update_stats(&test, &outcome, &mut self.failed_list);
            writer.print_test_post(test.name, outcome);
        }
    }

    fn run_rust_test(&self, test: &RustTestCase, ctx: &CaseContext) -> CaseOutcome {
        if !test.should_run_skip(self.config.disallow_skip()) {
            return CaseOutcome::Skipped;
        }

        // Explicit type to prevent tests from returning a value
        let err_context = || format!("gditest `{}` failed", test.name);
        let success: Option<()> =
            godot::private::handle_panic(err_context, || (test.function)(ctx));

        CaseOutcome::from_bool(success.is_some())
    }

    fn run_rust_benchmarks(&mut self, benchmarks: &mut GdBenchmarks) {
        let ctx = CaseContext {
            scene_tree: self.base.clone().upcast(),
        };

        let writer = MessageWriter::new();

        let mut first_line = " ".repeat(36).to_string();
        for metrics in BenchResult::metrics() {
            first_line.push_str(&format!("{:>13}", metrics));
        }
        writer.println(&first_line);

        let mut last_file = None;
        while let Some(bench) = benchmarks.get_benchmark() {
            writer.print_bench_pre(&bench, &mut last_file);
            let result = self.run_rust_benchmark(&bench, &ctx);
            self.benches_summary
                .update_stats(&bench, &result.outcome, &mut self.failed_list);
            writer.print_bench_post(bench.get_case_name(), result);
        }
    }

    fn run_rust_benchmark(&self, bench: &RustBenchmark, ctx: &CaseContext) -> BenchResult {
        if !bench.should_run_skip(self.config.disallow_skip()) {
            return BenchResult::skipped();
        }

        let inner_repetitions = bench.repetitions;

        // Explicit type to prevent bench from returning a value
        let err_context = || format!("gbbench `{}` failed", bench.name);

        let mut success: Option<()>;
        for _ in 0..crate::registry::bench::WARMUP_RUNS {
            success = godot::private::handle_panic(err_context, || (bench.function)(ctx));
            if success.is_none() {
                return BenchResult::failed();
            }
        }

        let mut times = Vec::with_capacity(501);
        for _ in 0..crate::registry::bench::TEST_RUNS {
            let start = Instant::now();
            success = godot::private::handle_panic(err_context, || (bench.function)(ctx));
            let duration = start.elapsed();
            if success.is_none() {
                return BenchResult::failed();
            }
            times.push(duration / inner_repetitions as u32);
        }
        times.sort();

        BenchResult::success(times)
    }
}
