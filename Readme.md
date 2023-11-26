# godot-test

This crate enhances the testing capabilities of [godot-rust](https://github.com/godot-rust/gdext) projects, enabling unit 
and integration testing as well as benchmarking.

When using the standard `#[test]` Rust macro in a Godot project, tests may fail if they involve objects requiring the Godot 
executable to run. `godot-test` addresses this limitation by introducing the `#[gditest]` and `#[gdbench]` macros. These 
can be used to annotate functions intended for testing or benchmarking your code, and the crate provides the `GdTestRunner` 
for executing them within a Godot scene.

## In Development

⚠️ **This crate is not production-ready.**

The API is still in early development and may undergo changes. Contributions, discussions, and feedback are highly encouraged.

## Output

Every execution of `GdTestRunner`` generates output, appearing in the terminal when run from the command line in headless mode 
or in the Godot console when executed from the editor. The output resembles the example below:

```
--------------------------------------------------------------------------------
--------------------           Running godot-test           --------------------
--------------------------------------------------------------------------------
                           Began run in HEADLESS mode                           

                              disallowing focused                               

   Found 8 Rust tests in 1 files
   Found 3 Rust benchmarks in 1 files

--------------------------------------------------------------------------------
   Running Rust tests
--------------------------------------------------------------------------------

   itest.rs:
   -- test_with_ctx ... ok!
   -- skipped_test ... ~skipped~
   -- simple_test ... ok!
   -- second_test ... ok!
   -- keyword_class_test ... ok!
   -- focused_test ... ok!
   -- filter_me_too ... ok!
   -- filter_me ... ok!

Test result: ok!. 7 passed; 0 failed, 1 skipped.
  Time: 0.00s.

--------------------------------------------------------------------------------
   Running Rust benchmarks
--------------------------------------------------------------------------------
                                              min       median

   bench.rs:
   -- skipped_bench              ...    ~skipped~
   -- focused_bench              ...      0.017μs      0.022μs
   -- bench_with_ctx             ...      2.425μs      4.056μs

Test result: ok!. 2 passed; 0 failed, 1 skipped.
  Time: 0.33s.

= = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = 
 = = = = = = = = = =             ! SUCCESS !               = = = = = = = = = =
= = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = 

  Took total: 0:00
```

## Setup

To run tests and benchmarks, the macros alone are not sufficient. You need to create a Godot project linked to your `gdext` 
extension. This project should contain a scene with a `GdTestRunner` node as the base node. You can easily set this up when 
creating a Godot application. If you're developing a `gdext` extension, consider having a minimal Godot project inside your 
crate's workspace. You can find example of the implementation in the `tests` subcrate of this crate.

After setting up the scene, you can run it from the Godot editor or the command line. Refer to the `GdTestRunner` documentation 
for additional information.

## Note

The functionality of this crate is heavily inspired by the [internal tests of `godot-rust`](https://github.com/godot-rust/gdext/tree/master/itest).
