/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use proc_macro::TokenStream;
use utils::translate_meta;

mod bench;
mod itest;
pub(crate) mod parser;
mod utils;

/// Integration test between Godot and Rust.
///
/// Similar to `#[test]`, but converts the function into an integration test between Godot and Rust.
///
/// It transforms and registers the annotated function for further usage by [`GdTestRunner`](gd_rehearse_defs::runner::GdTestRunner)
/// in some Godot test scene. When the runner enters the scene, it will run all qualified tests.
///
/// A function annotated with `#[gditest]` needs to:
/// - Have no return values.
/// - Have no parameters or only a singular [`CaseContext`](gd_rehearse_defs::cases::CaseContext).
///
/// ## Attributes
/// An attribute-less macro will make the tests run, but some attributes are available for better customizability, especially when working
/// on specific attributes and creating more narrow test runner scenes.
///
/// - `skip`: Skips the test during run.
/// - `focus`: Forces focus run, in which only tests annotated with `focus` will be run.
/// - `keyword`: A specific keyword that will be picked up by the runner, and the test will be run only if the runner has the same keyword specified.
/// - `scene_path`: Godot path to the scene. If specified, given benchmark will only run if runner's scene path is the same.
///
/// ## Examples
/// ```no_run
/// use gd_rehearse::CaseContext;
/// use gd_rehearse::itest::*;
///
/// // Causes a focus run during which only the focused tests will be executed, but only with
/// // `my test` as a keyword in the runner.
/// #[gditest(focus, keyword = "my test")]
/// fn focused_test() {
///     assert!(true);
/// }
///
/// // Will be skipped during the default run.
/// #[gditest(skip)]
/// fn skipped_test() {
///     let test = 1 + 1;
///     assert_eq!(test, 1);
/// }
///
/// // Can access the `GdTestRunner` scene_tree.
/// #[gditest]
/// fn test_with_ctx(ctx: &CaseContext) {
///     ctx.scene_tree().instance_id();
/// }
///
/// // Will only run in `res://special_cases.tscn` scene.
/// #[gditest(scene_path="res://special_cases.tscn")]
/// fn test_with_path(ctx: &CaseContext) {
///     let test_node = ctx.scene_tree().get_node("SomeTestNode".into());
///     assert!(test_node.is_some());
///     assert!(!test_node.unwrap().get("property_should_be_here".into()).is_nil());
/// }  
/// ```
#[proc_macro_attribute]
pub fn gditest(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gditest", meta, input, itest::attribute_gditest)
}

/// Benchmark for gdext classes and functions integrated with Godot.
///
/// This macro transforms and registers the annotated function for further usage by [`GdTestRunner`](gd_rehearse_defs::runner::GdTestRunner)
/// within a Godot test scene. When the runner enters the scene, it executes all qualified benchmarks.
///
/// A function annotated with `#[gdbench]` must:
/// - Have a return value.
/// - Have no parameters or only a singular [`CaseContext`](gd_rehearse_defs::cases::CaseContext).
///
/// Every benchmark is executed 200 times for a *warm-up*, followed by 501 additional runs to assess runtime (an odd number of runs for easy
/// median extraction). Minimum and median run times will be displayed.
///
/// ## Attributes
/// An attribute-less macro will make the benchmark run, but several attributes are available for better customizability, especially when
/// working on specific attributes and creating more narrowly-focused test runner scenes.
///
/// - `skip`: Skips the benchmark during execution.
/// - `focus`: Forces a focused run, in which only benchmarks annotated with `focus` will be executed.
/// - `keyword`: A specific keyword that will be picked up by the runner. The benchmark runs only if the runner has the same keyword specified.
/// - `scene_path`: Godot path to the scene. If specified, given benchmark will only run if runner's scene path is the same.
/// - `repeat`: Specifies the number of internal repeats the benchmark should undergo. By default, the function executes 100 times within every run.
///
/// ## Examples
/// ```no_run
/// use gd_rehearse::CaseContext;
/// use gd_rehearse::bench::*;
/// use godot::obj::InstanceId;
/// use godot::builtin::Variant;
///
/// // Causes a focus run during which only the focused benchmarks will be executed, but only with
/// // `my bench` as a keyword in the runner.
/// #[gdbench(focus, keyword = "my bench")]
/// fn focused_bench() -> i32 {
///     1337
/// }
///
/// // Will be skipped during the default run.
/// #[gdbench(skip)]
/// fn skipped_test() -> i32 {
///     231 + 312
/// }
///
/// // Can access the `GdTestRunner` scene_tree.
/// #[gdbench]
/// fn bench_with_ctx(ctx: &CaseContext) -> InstanceId {
///     ctx.scene_tree().instance_id()
/// }
/// 
/// // Will only run in `res://special_cases.tscn` scene.
/// #[gdbench(scene_path="res://special_cases.tscn")]
/// fn bench_with_path(ctx: &CaseContext) -> Variant {
///     let test_node = ctx.scene_tree().get_node("SomeTestNode".into()).expect("Can't get node");
///     let variant = test_node.get("property_should_be_here".into());
///     assert!(!variant.is_nil());
///     variant
/// } 
/// ```
#[proc_macro_attribute]
pub fn gdbench(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gdbench", meta, input, bench::attribute_bench)
}
