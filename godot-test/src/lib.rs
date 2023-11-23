pub use godot_test_defs::*;
pub use godot_test_macros::*;

pub use godot_test_defs::cases::CaseContext;

pub mod itest {
    pub use godot_test_defs::cases::rust_test_case::RustTestCase;
    pub use godot_test_defs::registry::itest::*;
    pub use godot_test_macros::gditest;
}

pub mod bench {
    pub use godot_test_defs::cases::rust_bench::{bench_used, RustBenchmark};
    pub use godot_test_defs::registry::bench::*;
    pub use godot_test_macros::gdbench;
}
