/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::collections::HashSet;
use std::time::Duration;

use crate::cases::rust_bench::RustBenchmark;
use crate::cases::{Case, CaseOutcome};
use crate::runner::config::RunnerConfig;

pub(crate) const WARMUP_RUNS: usize = 200;
pub(crate) const TEST_RUNS: usize = 501; // uneven, so median need not be interpolated.
const METRIC_COUNT: usize = 2;

godot::sys::plugin_registry!(pub GODOT_TEST_RUST_BENCHMARKS: RustBenchmark);

#[doc(hidden)]
pub(crate) struct GdBenchmarks {
    benches: Vec<RustBenchmark>,
    files_count: usize,
    is_focus_run: bool,
}

impl GdBenchmarks {
    pub fn bench_count(&self) -> usize {
        self.benches.len()
    }

    pub fn files_count(&self) -> usize {
        self.files_count
    }

    pub(crate) fn init(config: &RunnerConfig, is_focus_run: bool) -> Self {
        let mut instance = Self {
            benches: Vec::new(),
            files_count: 0,
            is_focus_run,
        };

        instance.collect_rust_benchmarks(config);
        instance
    }

    pub fn get_post_init_summary(&self) -> String {
        format!(
            "   Found {} Rust benchmarks in {} files",
            self.bench_count(),
            self.files_count()
        )
    }

    pub fn get_benchmark(&mut self) -> Option<RustBenchmark> {
        self.benches.pop()
    }

    fn get_benchmark_from_registry() -> Option<RustBenchmark> {
        __godot_rust_plugin_GODOT_TEST_RUST_BENCHMARKS
            .lock()
            .expect("couldn't retrieve RustBenchmark")
            .pop()
    }

    fn collect_rust_benchmarks(&mut self, config: &RunnerConfig) {
        let mut all_files = HashSet::new();

        while let Some(bench) = Self::get_benchmark_from_registry() {
            // Collect only benches based on keyword. If keyword in runner is empty, all will pass this check
            if bench.should_run_keyword(config.keyword(), config.ignore_keywords()) {
                if !self.is_focus_run && bench.is_case_focus() && !config.disallow_focus() {
                    self.benches.clear();
                    all_files.clear();
                    self.is_focus_run = true;
                }

                if (!self.is_focus_run && bench.should_run_filters(config.filters()))
                    || bench.should_run_focus(config.disallow_focus())
                {
                    all_files.insert(bench.file);
                    self.benches.push(bench);
                }
            }
        }

        // Sort for deterministic run order: by file name and line number.
        self.benches
            .sort_by(|a, b| format!("{}{}", b.file, b.line).cmp(&format!("{}{}", a.file, a.line)));

        self.files_count = all_files.len();
    }
}

pub(crate) struct BenchResult {
    pub outcome: CaseOutcome,
    pub stats: [Duration; METRIC_COUNT],
}

impl BenchResult {
    pub fn skipped() -> Self {
        Self {
            outcome: CaseOutcome::Skipped,
            stats: [Duration::ZERO, Duration::ZERO],
        }
    }

    pub fn failed() -> Self {
        Self {
            outcome: CaseOutcome::Failed,
            stats: [Duration::ZERO, Duration::ZERO],
        }
    }

    pub fn metrics() -> [&'static str; METRIC_COUNT] {
        ["min", "median"]
    }

    pub fn success(times: Vec<Duration>) -> BenchResult {
        // See top of file for rationale.

        /*let mean = {
            let total = times.iter().sum::<Duration>();
            total / TEST_RUNS as u32
        };
        let std_dev = {
            let mut variance = 0;
            for time in times.iter() {
                let diff = time.as_nanos() as i128 - mean.as_nanos() as i128;
                variance += (diff * diff) as u128;
            }
            Duration::from_nanos((variance as f64 / TEST_RUNS as f64).sqrt() as u64)
        };
        let max = times[TEST_RUNS - 1];
        let percentile05 = times[(TEST_RUNS as f64 * 0.05) as usize];
        */

        // Interpolating percentiles is not that important.
        let min = times[0];
        let median = times[TEST_RUNS / 2];

        BenchResult {
            outcome: CaseOutcome::Passed,
            stats: [min, median],
        }
    }
}
