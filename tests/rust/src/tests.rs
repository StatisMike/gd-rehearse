use godot_test::{bench::GdBenchmarks, gdbench, itest::*};
#[test]
fn gditest_macro_can_be_used() {
    #[gditest]
    fn empty_test() {}

    // Need to empty the tests static
    _ = GdRustItests::init(&[]);
}

#[test]
fn gditest_macro_can_be_used_with_context() {
    #[gditest]
    fn empty_test_with_ctx(ctx: &TestContext) {
        let _ = &ctx.property_tests;
        let _ = &ctx.scene_tree;
    }

    // Need to empty the tests static
    _ = GdRustItests::init(&[]);
}

#[test]
fn gditest_can_be_registered() {
    #[gditest]
    fn empty_test() {}
    let test_line = std::line!() - 2;

    let mut registered_tests = GdRustItests::init(&[]);

    assert_eq!(registered_tests.files_count(), 1);
    assert!(!registered_tests.is_focus_run());

    let only_case = registered_tests.get_test();

    assert!(only_case.is_some());

    let only_case = only_case.unwrap();

    assert_eq!(only_case.file, std::file!());
    assert_eq!(only_case.line, test_line);
    assert_eq!(only_case.name, "empty_test");
    assert!(!only_case.skipped);
    assert!(!only_case.focused);

    let no_more_case = registered_tests.get_test();

    assert!(no_more_case.is_none());
}

#[test]
fn gditests_can_be_skipped() {
    #[gditest(skip)]
    fn skipped_test() {}

    #[gditest]
    fn normal_test() {}

    let mut registered_tests = GdRustItests::init(&[]);

    assert!(!registered_tests.is_focus_run());

    let skipped_test = registered_tests.get_test().unwrap();

    assert!(skipped_test.skipped);

    let normal_test = registered_tests.get_test().unwrap();

    assert!(!normal_test.skipped);

    assert!(registered_tests.get_test().is_none());
}

#[test]
fn gditests_can_be_focused() {
    #[gditest(focus)]
    fn focused_test() {}

    #[gditest]
    fn normal_test() {}

    let mut registered_tests = GdRustItests::init(&[]);

    assert!(registered_tests.is_focus_run());

    let focused_test = registered_tests.get_test().unwrap();

    assert!(focused_test.focused);

    assert!(registered_tests.get_test().is_none());
}

#[test]
fn gdbench_macro_can_be_used() {
    #[gdbench]
    fn empty_bench() -> usize {
        2
    }

    // Need to empty the bench static
    _ = GdBenchmarks::init();
}

#[test]
fn gdbench_can_be_registered() {
    #[gdbench]
    fn empty_bench() -> usize {
        2
    }
    let test_line = std::line!() - 2;

    let mut registered_benches = GdBenchmarks::init();

    assert_eq!(registered_benches.files_count(), 1);

    let only_case = registered_benches.get_benchmark();

    assert!(only_case.is_some());

    let only_case = only_case.unwrap();

    assert_eq!(only_case.file, std::file!());
    assert_eq!(only_case.line, test_line);
    assert_eq!(only_case.name, "empty_bench");
    assert_eq!(only_case.repetitions, 100);

    let no_more_case = registered_benches.get_benchmark();

    assert!(no_more_case.is_none());
}

#[test]
fn gdbench_with_reps_can_be_registered() {
    #[gdbench(repeat = 250)]
    fn empty_bench() -> usize {
        2
    }
    let test_line = std::line!() - 2;

    let mut registered_benches = GdBenchmarks::init();

    assert_eq!(registered_benches.files_count(), 1);

    let only_case = registered_benches.get_benchmark();

    assert!(only_case.is_some());

    let only_case = only_case.unwrap();

    assert_eq!(only_case.file, std::file!());
    assert_eq!(only_case.line, test_line);
    assert_eq!(only_case.name, "empty_bench");
    assert_eq!(only_case.repetitions, 250);

    let no_more_case = registered_benches.get_benchmark();

    assert!(no_more_case.is_none());
}
