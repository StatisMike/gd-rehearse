use godot::init::{gdextension, ExtensionLibrary};

mod itest;

struct GodotTestTests;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Entry point

#[gdextension(entry_point=tests_init)]
unsafe impl ExtensionLibrary for GodotTestTests {}
