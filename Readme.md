# gd-rehearse
![Tests](https://github.com/StatisMike/gd-rehearse/actions/workflows/tests.yaml/badge.svg)
![godot-rust v0.2.4](https://img.shields.io/badge/godot--rust-v0.2.4-blue?style=plastic)

>In the vibrant production of game development, testing serves as the ongoing rehearsals where every line of code steps onto the stage. Just as actors grasping for fame polish their performances before the grand opening night, game makers of all calibers should fine-tune their creation, ensuring a flawless gameplay experience when the curtain rises on the final release.

This crate enhances the testing capabilities of [godot-rust](https://github.com/godot-rust/gdext) projects, enabling unit 
and integration testing as well as benchmarking.

When using the standard `#[test]` Rust macro in a gdextension project, tests will fail if they involve objects that require the Godot executable to run. `gd-rehearse` provides a suitable testing stage by introducing the `#[gditest]` and `#[gdbench]` macros. These can be used to annotate functions intended for testing or benchmarking your code, and the crate provides the GdTestRunner for executing them within a created Godot test scene.

## In Development

⚠️ **This crate is not production-ready.**

The API is still in early development and may undergo changes. Contributions, discussions, and feedback are highly encouraged.

## Output

Every execution of `GdTestRunner` generates output, appearing in the terminal when run from the command line in headless mode 
or in the Godot console when executed from the editor. The output resembles the example below:

```
$ godot --path tests/godot/ --headless
Initialize godot-rust (API v4.2.stable.official, runtime v4.2.stable.official)
Godot Engine v4.2.stable.official.46dc27791 - https://godotengine.org

--------------------------------------------------------------------------------
--------------------          Running gd-rehearse           --------------------
--------------------------------------------------------------------------------
              Began run in HEADLESS mode in scene: res://test.tscn              
                     Rust build: debug; Godot build: debug                      
                              disallowing focused                               

   Found 6 Rust tests in 1 files
   Found 3 Rust benchmarks in 1 files

--------------------------------------------------------------------------------
   Running Rust tests
--------------------------------------------------------------------------------

   itest.rs:
   -- simple_test ... ok!
   -- second_test ... ok!
   -- focused_test ... ok!
   -- skipped_test ... ~skipped~
   -- filter_me ... ok!
   -- filter_me_too ... ok!

Tests result: ok! 5 passed; 0 failed, 1 skipped. Elapsed: 0.00s.

--------------------------------------------------------------------------------
   Running Rust benchmarks
--------------------------------------------------------------------------------
                                              min       median
   bench.rs:
   -- focused_bench              ...      0.015μs      0.016μs
   -- skipped_bench              ...    ~skipped~
   -- normal_bench               ...      0.016μs      0.020μs

Benchmarks result: ok! 2 passed; 0 failed, 1 skipped. Elapsed: 0.00s.

= = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = 
 = = = = = = = = = =             ! SUCCESS !               = = = = = = = = = =
= = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = =
```

Or for fans of straight to the point, commandline option `--quiet-run` can be used:

```
$ godot --path tests/godot/ --headless -- --quiet-run
Initialize godot-rust (API v4.2.stable.official, runtime v4.2.stable.official)
Godot Engine v4.2.stable.official.46dc27791 - https://godotengine.org
 
Tests result: ok! 5 passed; 0 failed, 1 skipped. Elapsed: 0.00s.
Benchmarks result: ok! 2 passed; 0 failed, 1 skipped. Elapsed: 0.00s.
```

## Setup

To run tests and benchmarks, the macros alone are not sufficient. You need to create a Godot project using your `gdext` 
extension. This project should contain a scene with a `GdTestRunner` node as the base node - so additional scene if you 
are creating a Godot application. If you're developing a `gdext` extension, you need to have a minimal Godot project inside your 
crate's workspace. You can find an example of the implementation in the `tests` directory of this repo.

After setting up the scene, you can run it from the Godot editor or the command line. Refer to the `GdTestRunner` documentation 
for additional information.

> ⚠️ While running tests from the editor, if the full runner run is very short the output won't always get printed to Godot
console.

## GitHub CI hints

To setup `gd-rehearse` in Github Actions, remember to include `.godot/extension_list.cfg` at minimal inside your Godot project. 
Even though Godot 4+ includes whole `.godot` directory as ignored, without this file commandline Godot run won't be able to 
find your `*.gdextension` file, thus not loading the `GdTestRunner`. 

## Note

The functionality of this crate is heavily inspired by the [internal tests of `godot-rust`](https://github.com/godot-rust/gdext/tree/master/itest), 
and core portions of source code are borrowed from there. Big thanks to maintainers and contributors of `godot-rust`!