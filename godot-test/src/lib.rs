/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

pub use godot_test_defs::cases::CaseContext;
pub use godot_test_defs::runner::GdTestRunner;
pub use godot_test_macros::*;

pub mod itest {
    pub use godot_test_defs::cases::rust_test_case::RustTestCase;
    pub use godot_test_defs::registry::itest::*;
}

pub mod bench {
    pub use godot_test_defs::cases::rust_bench::{bench_used, RustBenchmark};
    pub use godot_test_defs::registry::bench::*;
}
