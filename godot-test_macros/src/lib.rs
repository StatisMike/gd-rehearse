/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use venial::Declaration;

mod bench;
mod itest;
pub(crate) mod parser;
mod utils;

/// Integration test between Godot and Rust.
///
/// Similar to `#[test]`, but converts the function into an integration test between Godot and Rust.
///
/// It transforms and registers the annotated function for further usage by [`GdTestRunner`](godot_test_defs::runner::GdTestRunner)
/// in some Godot test scene. When the runner enters the scene, it will run all qualified tests.
///
/// A function annotated with `#[gditest]` needs to:
/// - Have no return values.
/// - Have no parameters or only a singular [`CaseContext`](godot_test_defs::cases::CaseContext).
///
/// ## Attributes
/// An attribute-less macro will make the tests run, but some attributes are available for better customizability, especially when working
/// on specific attributes and creating more narrow test runner scenes.
///
/// - `skip`: Skips the test during run.
/// - `focus`: Forces focus run, in which only tests annotated with `focus` will be run.
/// - `keyword`: A specific keyword that will be picked up by the runner, and the test will be run only if the runner has the same keyword specified.
///
/// ## Examples
/// ```no_run
/// use godot_test::*;
///
/// // Will cause a focus run during which only the focused test will be run, but only with `my test` as a keyword in the runner.
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
///     ctx.scene_tree.instance_id();
/// }
/// ```
#[proc_macro_attribute]
pub fn gditest(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gditest", meta, input, itest::attribute_gditest)
}

/// Benchmark for gdext classes and functions integrated with Godot.
///
/// It transforms and registers the annotated function for further usage by [`GdTestRunner`](godot_test_defs::runner::GdTestRunner)
/// in some Godot test scene. When the runner enters the scene, it will run all qualified benchmarks.
///
/// A function annotated with `#[gdbench]` needs to:
/// - Have a return value.
/// - Have no parameters or only a singular [`CaseContext`](godot_test_defs::cases::CaseContext).
///
/// Every benchmark is run 200 times for a *warm-up*, and then 501 times to assess runtime (uneven times to easily extract the median).
/// Minimum and median run times will be shown.
///
/// ## Attributes
/// An attribute-less macro will make the benchmark run, but there are some available for better customizability, especially when working on
/// specific attributes and creating more narrow test runner scenes.
///
/// - `skip`: Skips the benchmark during run.
/// - `focus`: Forces focus run, in which only benchmarks annotated with `focus` will be run.
/// - `keyword`: A specific keyword that will be picked up by the runner, and the benchmark will be run only if the runner has the same keyword specified.
/// - `repeat`: Number of repeats the benchmark should be run internally. By default, the function will execute 100 times within every run.
///
/// ## Examples
/// ```no_run
/// use godot_test::*;
/// use godot::obj::InstanceId;
///
/// // Will cause a focus run during which only the focused test will be run, but only with `my bench` as a keyword in the runner.
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
///     ctx.scene_tree.instance_id()
/// }
/// ```
#[proc_macro_attribute]
pub fn gdbench(meta: TokenStream, input: TokenStream) -> TokenStream {
    translate_meta("gdbench", meta, input, bench::attribute_bench)
}

fn translate_meta<F>(
    self_name: &str,
    meta: TokenStream,
    input: TokenStream,
    transform: F,
) -> TokenStream
where
    F: FnOnce(Declaration) -> Result<TokenStream2, venial::Error>,
{
    let self_name = format_ident!("{}", self_name);
    let input2 = TokenStream2::from(input);
    let meta2 = TokenStream2::from(meta);

    // Hack because venial doesn't support direct meta parsing yet
    let input = quote! {
        #[#self_name(#meta2)]
        #input2
    };

    let result2 = venial::parse_declaration(input)
        .and_then(transform)
        .unwrap_or_else(|e| e.to_compile_error());

    TokenStream::from(result2)
}
