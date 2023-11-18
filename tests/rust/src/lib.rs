use godot::init::{gdextension, ExtensionLibrary};

mod itest;
#[cfg(test)]
mod tests;

struct GodotTestTests;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Entry point

#[gdextension(entry_point=tests_init)]
unsafe impl ExtensionLibrary for GodotTestTests {}
