# godot-test

This crate enhances the testing capabilities of [godot-rust](https://github.com/godot-rust/gdext) projects, enabling unit and integration testing as well as benchmarking.

When using the standard `#[test]` Rust macro in a Godot project, tests may fail if they involve objects requiring the Godot executable to run. `godot-test` addresses this limitation by introducing the `#[gditest]` and `#[gdbench]` macros. These can be used to annotate functions intended for testing or benchmarking your code, and the crate provides the `GdTestRunner` for executing them within a Godot scene.

## In Development

⚠️ **This crate is not production-ready.**

The API is still in early development and may undergo changes. Contributions, discussions, and feedback are highly encouraged.

## Setup

To run tests and benchmarks, the macros alone are not sufficient. You need to create a Godot project linked to your `gdext` extension. This project should contain a scene with a `GdTestRunner` node as the base node. You can easily set this up when creating a Godot application. If you're developing a `gdext` extension, consider having a minimal Godot project inside your crate's workspace. You can find examples in the `tests` subcrate of this crate.

After setting up the scene, you can run it from the Godot editor or the command line. Refer to the `GdTestRunner` documentation for additional information.

## Note

The functionality of this crate is inspired by the [internal tests of `godot-rust`](https://github.com/godot-rust/gdext/tree/master/itest).
