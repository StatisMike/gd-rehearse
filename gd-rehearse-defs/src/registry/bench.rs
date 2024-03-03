/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::time::Duration;

use crate::cases::rust_bench::{BenchError, RustBenchmark};
use crate::cases::CaseOutcome;

use super::CaseFilterer;

pub(crate) const WARMUP_RUNS: usize = 200;
pub(crate) const TEST_RUNS: usize = 501; // uneven, so median need not be interpolated.
const METRIC_COUNT: usize = 2;

godot::sys::plugin_registry!(pub GD_REHEARSE_RUST_BENCHMARKS: RustBenchmark);

#[doc(hidden)]
pub(crate) struct GdBenchmarks {
    benches: Vec<RustBenchmark>,
    files_count: usize,
    is_focus_run: bool,
    is_path_run: bool,
}

impl GdBenchmarks {
    pub fn bench_count(&self) -> usize {
        self.benches.len()
    }

    pub fn files_count(&self) -> usize {
        self.files_count
    }

    pub(crate) fn init() -> Self {
        let mut instance = Self {
            benches: Vec::new(),
            files_count: 0,
            is_focus_run: false,
            is_path_run: false,
        };

        instance.collect_rust_benchmarks();
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
        __godot_rust_plugin_GD_REHEARSE_RUST_BENCHMARKS
            .lock()
            .expect("couldn't retrieve RustBenchmark")
            .pop()
    }

    fn collect_rust_benchmarks(&mut self) {
        while let Some(bench) = Self::get_benchmark_from_registry() {
            self.benches.push(bench);
        }
    }

    pub fn finish_setup(&mut self) {
        self.sort_cases();
        self.files_count = self.get_files_count()
    }
}

pub(crate) struct BenchResult {
    pub outcome: CaseOutcome,
    pub stats: [Duration; METRIC_COUNT],
    pub error: Option<BenchError>,
}

impl BenchResult {
    pub fn skipped() -> Self {
        Self {
            outcome: CaseOutcome::Skipped,
            stats: [Duration::ZERO, Duration::ZERO],
            error: None,
        }
    }

    pub fn failed(err: BenchError) -> Self {
        Self {
            outcome: CaseOutcome::Failed,
            stats: [Duration::ZERO, Duration::ZERO],
            error: Some(err),
        }
    }

    pub fn metrics() -> [&'static str; METRIC_COUNT] {
        ["min", "median"]
    }

    pub fn success(times: Vec<Duration>) -> BenchResult {
        // Currently more metrics unused, as in `gdext/itest`

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
            error: None,
        }
    }
}

impl CaseFilterer<RustBenchmark> for GdBenchmarks {
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
    fn get_cases(&self) -> &Vec<RustBenchmark> {
        &self.benches
    }
    fn get_cases_mut(&mut self) -> &mut Vec<RustBenchmark> {
        &mut self.benches
    }
}
