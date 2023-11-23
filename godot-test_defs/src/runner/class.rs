use std::time::{Duration, Instant};

use godot::prelude::{godot_api, Base, GString, GodotClass, INode, Node, PackedStringArray};

use crate::{
    cases::{
        rust_bench::RustBenchmark, rust_test_case::RustTestCase, Case, CaseContext, CaseOutcome,
    },
    registry::{
        bench::{BenchResult, GdBenchmarks},
        itest::GdRustItests,
    },
};

use super::{config::RunnerConfig, extract_file_subtitle, print::MessageWriter};

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
            "\nTest result: {outcome}. {passed} passed; {failed} failed{extra}."
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

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GdTestRunner {
    #[export]
    disallow_focus: bool,
    #[export]
    disallow_skip: bool,
    #[export]
    test_filters: PackedStringArray,
    #[export]
    test_keyword: GString,
    #[export]
    run_benchmarks: bool,
    #[export]
    run_tests: bool,
    tests_summary: RunnerSummary,
    benches_summary: RunnerSummary,
    config: RunnerConfig,
    failed_list: Vec<String>,
    focus_run: bool,
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
            run_benchmarks: true,
            run_tests: true,
            tests_summary: RunnerSummary::default(),
            benches_summary: RunnerSummary::default(),
            config: RunnerConfig::default(),
            failed_list: Vec::new(),
            focus_run: true,
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
        let writer = MessageWriter::new();
        writer.print_begin();

        self.config = RunnerConfig::new(
            self.disallow_focus,
            self.disallow_skip,
            self.run_tests,
            self.run_benchmarks,
            &self.test_keyword,
            &self.test_filters,
        )
        .unwrap();

        self.config.print_info();

        let mut rust_test_outcome = true;
        let mut rust_bench_outcome = true;

        let mut rust_tests_handler: Option<GdRustItests> = None;
        let mut rust_bench_handler: Option<GdBenchmarks> = None;

        // Gather tests and benches to run based on the config.
        if self.config.run_rust_tests() {
            let handler = GdRustItests::init(&self.config);
            writer.println(&handler.get_post_init_summary());
            rust_tests_handler = Some(handler);
        }

        if self.config.run_rust_benchmarks() {
            let handler = GdBenchmarks::init(&self.config);
            writer.println(&handler.get_post_init_summary());
            rust_bench_handler = Some(handler);
        }

        // Run Rust Tests.
        if let Some(mut handler) = rust_tests_handler {
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

        if outcome {
          writer.print_success()
        } else {
          writer.print_failure()
        }

        self.base
            .get_tree()
            .unwrap()
            .quit_ex()
            .exit_code(outcome as i32)
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

        let mut first_line = format!("\n{space}", space = " ".repeat(36));
        for metrics in BenchResult::metrics() {
          first_line.push_str(&format!("{:>13}", metrics));
        }
        writer.println(&first_line);

        let mut last_file = None;
        if let Some(bench) = benchmarks.get_benchmark() {
            writer.print_bench_pre(&bench, &mut last_file);
            let result = self.run_rust_benchmark(&bench, &ctx);
            self.benches_summary.update_stats(&bench, &result.outcome, &mut self.failed_list);
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

        let mut success: Option<()> = None;
        for _ in 0..200 {
            success = godot::private::handle_panic(err_context, || (bench.function)(ctx));
            if success.is_none() {
                return BenchResult::failed();
            }
        }

        let mut times = Vec::with_capacity(501);
        for _ in 0..501 {
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
