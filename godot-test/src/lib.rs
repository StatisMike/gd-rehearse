pub use godot_test_defs::*;
pub use godot_test_macros::*;

pub mod itest {
    pub use godot_test_defs::classes::{RustTestCase, TestContext};
    pub use godot_test_defs::registry::itest::*;
    pub use godot_test_macros::gditest;
}

pub mod bench {

    pub use godot_test_defs::classes::{bench_used, RustBenchmark};
    pub use godot_test_defs::registry::bench::*;
}
