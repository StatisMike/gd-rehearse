use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::classes::RustBenchmark;

const WARMUP_RUNS: usize = 200;
const TEST_RUNS: usize = 501; // uneven, so median need not be interpolated.
const METRIC_COUNT: usize = 2;

static RUST_BENCHMARKS: std::sync::Mutex<Vec<RustBenchmark>> = std::sync::Mutex::new(Vec::new());

pub fn register_benchmark(case: RustBenchmark) {
    RUST_BENCHMARKS
        .lock()
        .expect("can't add RustBenchmark")
        .push(case);
}

pub struct GdBenchmarks {
    benches: Vec<RustBenchmark>,
    files_count: usize,
}

impl GdBenchmarks {
    pub fn bench_count(&self) -> usize {
        self.benches.len()
    }

    pub fn files_count(&self) -> usize {
        self.files_count
    }

    pub fn init() -> Self {
        let mut instance = Self {
            benches: Vec::new(),
            files_count: 0,
        };

        instance.collect_rust_benchmarks();
        instance
    }

    pub fn get_benchmark(&mut self) -> Option<RustBenchmark> {
        self.benches.pop()
    }

    fn get_benchmark_from_registry() -> Option<RustBenchmark> {
        RUST_BENCHMARKS
            .lock()
            .expect("couldn't retrieve RustBenchmark")
            .pop()
    }

    fn collect_rust_benchmarks(&mut self) {
        let mut all_files = HashSet::new();

        while let Some(bench) = Self::get_benchmark_from_registry() {
            self.benches.push(bench);
            all_files.insert(bench.file);
        }

        // Sort alphabetically for deterministic run order
        self.benches.sort_by_key(|bench| bench.file);
        self.files_count = all_files.len();
    }
}

pub struct BenchResult {
    pub stats: [Duration; METRIC_COUNT],
}

impl BenchResult {
    pub fn metrics() -> [&'static str; METRIC_COUNT] {
        ["min", "median"]
    }

    pub fn run_benchmark(code: fn(), inner_repetitions: usize) -> BenchResult {
        for _ in 0..WARMUP_RUNS {
            code();
        }

        let mut times = Vec::with_capacity(TEST_RUNS);
        for _ in 0..TEST_RUNS {
            let start = Instant::now();
            code();
            let duration = start.elapsed();

            times.push(duration / inner_repetitions as u32);
        }
        times.sort();

        Self::calculate_stats(times)
    }

    fn calculate_stats(times: Vec<Duration>) -> BenchResult {
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
            stats: [min, median],
        }
    }
}
