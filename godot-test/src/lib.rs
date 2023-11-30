/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
//! Framework for testing and benchmarking [`godot-rust`](godot)-dependent code.
//!
//! When using the standard `#[test]` Rust macro in a Godot project, tests may fail if they involve objects requiring the Godot
//! executable to run. `godot-test` addresses this limitation by introducing the [`#[gditest]`](macro@godot_test_macros::gditest) and
//! [`#[gdbench]`](macro@godot_test_macros::gdbench) macros. These can be used to annotate functions intended for testing or benchmarking
//! your code, and the crate provides the [`GdTestRunner`] [`GodotClass`](trait@godot::prelude::GodotClass) for executing them within
//! a Godot scene.

pub use godot_test_defs::cases::CaseContext;
pub use godot_test_defs::runner::GdTestRunner;

/// Contains all symbols necessary to use [`#[gditest]`](macro@godot_test_macros::gditest) macro.
pub mod itest {
    pub use godot_test_defs::cases::rust_test_case::RustTestCase;
    pub use godot_test_defs::registry::itest::*;
    pub use godot_test_macros::gditest;
}

/// Contains all symbols necessary to use [`#[gdbench]`](macro@godot_test_macros::gdbench) macro.
pub mod bench {
    pub use godot_test_defs::cases::rust_bench::{bench_used, RustBenchmark};
    pub use godot_test_defs::registry::bench::*;
    pub use godot_test_macros::gdbench;
}
