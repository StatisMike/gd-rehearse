/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use godot::obj::WithBaseField;
use godot::prelude::{godot_api, Base, GString, GodotClass, INode, Node, PackedStringArray};

use crate::cases::rust_bench::{BenchContext, BenchError, RustBenchmark};
use crate::cases::rust_test_case::{RustTestCase, TestContext};
use crate::cases::{Case, CaseOutcome, CaseType};

use crate::registry::bench::{BenchResult, GdBenchmarks};
use crate::registry::itest::{GdRustItests, TestResult};
use crate::registry::CaseFilterer;

use super::config::RunnerConfig;
use super::extract_file_subtitle;
use super::panic::UnwindError;
use super::print::MessageWriter;

use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct RunnerSummary {
    kind: CaseType,
    total: i64,
    passed: i64,
    skipped: i64,
}

impl RunnerSummary {
    pub fn new(kind: CaseType) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

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
            kind,
            total,
            passed,
            skipped,
        } = *self;

        let writer = MessageWriter::new(false);

        let kind_display = kind.for_summary();

        // Consider 0 cases run as a failure too, because it's probably a problem with the run itself.
        if total - skipped == 0 {
            writer.println(&format!("No {kind_display} cases were run. If this is intended, configure GdTestRunner to omit these cases."));
            return false;
        }

        let failed = total - passed - skipped;
        let all_passed = failed == 0;

        let outcome = CaseOutcome::from_bool(all_passed);

        let run_time = (run_time.as_secs_f32() * 100.).round() / 100.;

        let extra = if skipped > 0 {
            format!(", {skipped} skipped")
        } else {
            "".to_string()
        };

        writer.println(&format!(
            "{kind_display} result: {outcome} {passed} passed; {failed} failed{extra}. Elapsed: {run_time:.2}s."
        ));

        if !all_passed {
            writer.println("\n  Failed:");
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
///   will only be executed if all tests pass successfully.
/// - `test_keyword`: If set, only tests and benchmarks with the same `keyword` specified will be executed. Defaults to an empty string, meaning
///   that only tests and benchmarks without a `keyword` set will be executed. It takes precedence over `focus` and `filters`—they will be
///   assessed, but only in the context of this `keyword`.
/// - `ignore_keywords`: If set, all tests and benchmarks will be executed regardless of their set `keyword`.
/// - `disallow_focus`: If set, the `focus` attribute of tests and benchmarks will be ignored.
/// - `disallow_skip`: If set, the `skip` attribute of tests and benchmarks will be ignored.
/// - `test_filters`: An array of strings tested against the names of tests and benchmarks. Those with names containing at least one of the specified
///   filters will be executed.
/// - `only_scene_path`: If `true`, runner will execute only tests for its scene path specified in their `scene_path` attribute.
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
///   - `--only-scene-path`: Sets `only_scene_path` property with `true`
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
    #[export]
    only_scene_path: bool,
    tests_summary: RunnerSummary,
    benches_summary: RunnerSummary,
    config: RunnerConfig,
    failed_list: Vec<String>,
    began_run: bool,
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
            only_scene_path: false,
            tests_summary: RunnerSummary::new(CaseType::RustTest),
            benches_summary: RunnerSummary::new(CaseType::RustBenchmark),
            config: RunnerConfig::default(),
            failed_list: Vec::new(),
            began_run: false,
            base,
        }
    }
    // Needed for the physics to be initialized for the tests that needs them
    fn ready(&mut self) {
        let mut scene_tree = self.base().get_tree().unwrap();
        scene_tree.connect("physics_frame", &self.base().callable("test_run"));
    }
}

#[godot_api]
impl GdTestRunner {
    #[func]
    fn test_run(&mut self) {
        if self.began_run {
            return;
        }

        let path = self.base().get_scene_file_path().to_string();

        self.began_run = true;
        let writer = MessageWriter::new(false);

        match RunnerConfig::new(
            self.disallow_focus,
            self.disallow_skip,
            self.run_tests,
            self.run_benchmarks,
            &self.test_keyword,
            self.ignore_keywords,
            self.only_scene_path,
            path,
            &self.test_filters,
            false,
        ) {
            Ok(config) => self.config = config,
            Err(error) => {
                writer.println(&error.to_string());
                self.end(1);
                return;
            }
        }

        let writer = MessageWriter::new(self.config.is_quiet());

        writer.print_begin();

        writer.print_summary_info(&self.config);

        let mut rust_test_outcome = true;
        let mut rust_bench_outcome = true;

        let mut rust_tests_handler: Option<GdRustItests> = None;
        let mut rust_bench_handler: Option<GdBenchmarks> = None;

        let mut is_focus_run = false;

        // Gather tests and benches.
        if self.config.run_rust_tests() {
            let handler = GdRustItests::init();
            rust_tests_handler = Some(handler);
        }

        if self.config.run_rust_benchmarks() {
            let handler = GdBenchmarks::init();
            rust_bench_handler = Some(handler);
        }

        // Filter tests and benches on path and focus
        if let Some(handler) = &mut rust_tests_handler {
            handler.filter_path_keyword(&self.config);
        }
        if let Some(handler) = &mut rust_bench_handler {
            handler.filter_path_keyword(&self.config);
        }

        // Filter tests and benches on focus and filter
        if let Some(handler) = &mut rust_tests_handler {
            handler.set_focus_run(is_focus_run);
            handler.filter_focus_filters(&self.config);
            is_focus_run = handler.is_focus_run();
            handler.finish_setup();
            writer.println(&handler.get_post_init_summary());
        }
        if let Some(handler) = &mut rust_bench_handler {
            handler.set_focus_run(is_focus_run);
            handler.filter_focus_filters(&self.config);
            handler.finish_setup();
            // is_focus_run = handler.is_focus_run();
            writer.println(&handler.get_post_init_summary());
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

            writer.println("");
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

            writer.println("");
            rust_bench_outcome = self
                .benches_summary
                .conclude(run_time, &mut self.failed_list);
        }

        let outcome = rust_test_outcome && rust_bench_outcome;

        if outcome {
            writer.print_success()
        } else {
            writer.print_failure()
        }

        self.end(!outcome as i32);
    }

    fn end(&mut self, exit_code: i32) {
        self.base_mut().queue_free();
        self.base()
            .get_tree()
            .unwrap()
            .quit_ex()
            .exit_code(exit_code)
            .done();
    }

    fn run_rust_tests(&mut self, handler: &mut GdRustItests) {
        let ctx = TestContext::new(self.base().clone());

        let writer = MessageWriter::new(self.config.is_quiet());
        writer.println("");

        let mut last_file = None;
        while let Some(test) = handler.get_test() {
            writer.print_test_pre(test, &mut last_file);

            let result = self.run_rust_test(&test, &ctx);
            self.tests_summary
                .update_stats(&test, &result.outcome, &mut self.failed_list);
            writer.print_test_post(test.name, result);
        }
    }

    fn run_rust_test(&self, test: &RustTestCase, ctx: &TestContext) -> TestResult {
        if !test.should_run_skip(self.config.disallow_skip()) {
            return TestResult::skipped();
        }

        let result = super::panic::handle_panic(|| (test.function)(ctx));

        if let Err(err) = result {
            TestResult::failed(err)
        } else {
            TestResult::success()
        }
    }

    fn run_rust_benchmarks(&mut self, benchmarks: &mut GdBenchmarks) {
        let mut ctx = BenchContext::new(self.base().clone());

        let writer = MessageWriter::new(self.config.is_quiet());

        let mut first_line = " ".repeat(36).to_string();
        for metrics in BenchResult::metrics() {
            first_line.push_str(&format!("{:>13}", metrics));
        }
        writer.println(&first_line);

        let mut last_file = None;
        while let Some(bench) = benchmarks.get_benchmark() {
            writer.print_bench_pre(&bench, &mut last_file);

            let result = self.run_rust_benchmark(&bench, &mut ctx);

            self.benches_summary
                .update_stats(&bench, &result.outcome, &mut self.failed_list);
            writer.print_bench_post(bench.get_case_name(), result);
        }
    }

    fn run_rust_benchmark(&self, bench: &RustBenchmark, ctx: &mut BenchContext) -> BenchResult {
        if !bench.should_run_skip(self.config.disallow_skip()) {
            return BenchResult::skipped();
        }

        match bench.execute_setup_function(ctx.clone()) {
            Ok(setup_ctx) => *ctx = setup_ctx,
            Err(err) => {
                return BenchResult::failed(BenchError::Setup(err));
            }
        }

        let inner_repetitions = bench.repetitions;

        let mut success: Result<(), UnwindError>;
        for _ in 0..crate::registry::bench::WARMUP_RUNS {
            success = super::panic::handle_panic(|| (bench.function)(ctx));
            if let Err(err) = success {
                return BenchResult::failed(BenchError::Execution(err));
            }
        }

        ctx.zero_duration();

        let mut times = Vec::with_capacity(501);
        for _ in 0..crate::registry::bench::TEST_RUNS {
            let start = Instant::now();
            success = super::panic::handle_panic(|| (bench.function)(ctx));
            let duration = ctx.get_adjusted_duration(start);
            if let Err(err) = success {
                return BenchResult::failed(BenchError::Execution(err));
            }
            times.push(duration / inner_repetitions as u32);
        }
        times.sort();

        match bench.execute_cleanup_function(ctx.clone()) {
            Ok(cleanup_ctx) => *ctx = cleanup_ctx,
            Err(err) => return BenchResult::failed(BenchError::Cleanup(err)),
        }

        BenchResult::success(times)
    }
}
