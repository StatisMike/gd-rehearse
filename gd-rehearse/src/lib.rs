/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
//! Framework for testing and benchmarking [`godot-rust`](godot)-dependent code.
//!
//! When using the standard `#[test]` Rust macro in a Godot project, tests may fail if they involve objects requiring the Godot
//! executable to run. `godot-test` addresses this limitation by introducing the [`#[gditest]`](macro@gd_rehearse_macros::gditest) and
//! [`#[gdbench]`](macro@gd_rehearse_macros::gdbench) macros. These can be used to annotate functions intended for testing or benchmarking
//! your code, and the crate provides the [`GdTestRunner`] [`GodotClass`](trait@godot::prelude::GodotClass) for executing them within
//! a Godot scene.

pub use gd_rehearse_defs::cases::CaseContext;
pub use gd_rehearse_defs::runner::GdTestRunner;

/// Contains all symbols necessary to use [`#[gditest]`](macro@gd_rehearse_macros::gditest) macro.
pub mod itest {
    pub use gd_rehearse_defs::cases::rust_test_case::RustTestCase;
    pub use gd_rehearse_defs::registry::itest::*;
    pub use gd_rehearse_macros::gditest;
}

/// Contains all symbols necessary to use [`#[gdbench]`](macro@gd_rehearse_macros::gdbench) macro.
pub mod bench {
    pub use gd_rehearse_defs::cases::rust_bench::{bench_used, RustBenchmark};
    pub use gd_rehearse_defs::registry::bench::*;
    pub use gd_rehearse_macros::gdbench;
}
