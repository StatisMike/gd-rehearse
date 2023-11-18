use godot::{
    engine::{Engine, Node},
    obj::Gd,
};

pub struct TestContext {
    pub scene_tree: Gd<Node>,
    pub property_tests: Gd<Node>,
}

#[derive(Copy, Clone)]
pub struct RustTestCase {
    pub name: &'static str,
    pub file: &'static str,
    pub skipped: bool,
    /// If one or more tests are focused, only they will be executed. Helpful for debugging and working on specific features.
    pub focused: bool,
    #[allow(dead_code)]
    pub line: u32,
    pub function: fn(&TestContext),
}

#[derive(Copy, Clone)]
pub struct RustBenchmark {
    pub name: &'static str,
    pub file: &'static str,
    #[allow(dead_code)]
    pub line: u32,
    pub function: fn(),
    pub repetitions: usize,
}

#[derive(Copy, Clone)]
pub struct GodotTestCase {
    pub name: &'static str,
    pub file: &'static str,
}

/// Disable printing errors from Godot. Ideally we should catch and handle errors, ensuring they happen when
/// expected. But that isn't possible, so for now we can just disable printing the error to avoid spamming
/// the terminal when tests should error.
pub fn suppress_godot_print(mut f: impl FnMut()) {
    Engine::singleton().set_print_error_messages(false);
    f();
    Engine::singleton().set_print_error_messages(true);
}

/// Signal to the compiler that a value is used (to avoid optimization).
pub fn bench_used<T: Sized>(value: T) {
    // The following check would be used to prevent `()` arguments, ensuring that a value from the bench is actually going into the blackbox.
    // However, we run into this issue, despite no array being used: https://github.com/rust-lang/rust/issues/43408.
    //   error[E0401]: can't use generic parameters from outer function
    // sys::static_assert!(std::mem::size_of::<T>() != 0, "returned unit value in benchmark; make sure to use a real value");

    std::hint::black_box(value);
}
