use godot_test::bench::*;
use godot_test::gdbench;
use godot_test::gditest;
use godot_test::itest::*;
pub use godot_test::runner::GdTestRunner;

#[gditest]
fn simple_test() {
    let test = 1 + 1;
    assert_eq!(test, 2);
}

#[gditest]
fn second_test() {
    let test = 1 + 1;
    assert_eq!(test, 2);
}

#[gditest(focus)]
fn focused_test() {}

#[gditest(skip)]
fn skipped_test() {}

#[gditest(keyword = "my new class")]
fn keyword_class_test() {}

#[gdbench(keyword = "my new class")]
fn keyword_class_bench() -> i32 {
    243
}

#[gditest]
fn filter_me() {}

#[gditest]
fn filter_me_too() {}
