/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

// use godot::{bind::{GodotClass, godot_api}, builtin::{GString, StringName, PackedStringArray, Array}, log::{godot_error, godot_print}, engine::{GdScript, IRefCounted, RefCounted, Object, Resource}, obj::{Base, Gd}};

// #[derive(GodotClass)]
// #[class(base=RefCounted)]
// pub struct GdScriptCase {
//   #[var]
//   suite: Gd<Object>,
//   #[var]
//   assertion_failed: bool,
// }

// #[godot_api]
// impl IRefCounted for GdScriptCase {
//   fn init(_base: godot::obj::Base < Self::Base >) -> Self {
//       Self {
//         suite: Object::new_alloc(),
//         assertion_failed: false
//       }
//   }
// }

// impl GdScriptCase {
//   fn new_from_suite(suite: Gd<Object>) -> Self {
//     Self {
//       suite,
//       assertion_failed: false
//     }
//   }
// }

// #[godot_api]
// impl GdScriptCase {}

// #[derive(GodotClass)]
// #[class(init, base=Resource)]
// pub struct GdScriptTestSuite {
//   #[var]
//   assertion_failed: bool,
//   #[var]
//   suite_name: GString,
//   #[var]
//   test_names: Array<StringName>,
//   base: Base<Resource>
// }

// #[godot_api]
// impl GdScriptTestSuite {
//   // #[func]
//   pub fn get_file_path(&mut self) {
//     let script = self.base.path();
//     godot_print!("{}", script);
//   }
//   #[func]
//   pub fn gather_tests(&mut self) {
//     let methods = self.base.get_method_list();

//     for method in methods.iter_shared() {
//       let method_name = method.get("name").expect("can't get method name").to_string();
//       if !method_name.starts_with("test_") {
//         continue;
//       }
//       self.test_names.push(StringName::from(method_name));
//     }
//   }

//   // #[func]
//   // pub fn run_all_tests(&mut self) {
//   //   let tests_num = self.test_names.len();
//   //   for test_i in 0..tests_num {
//   //     let test_name = self.test_names.get(test_i).unwrap().clone();
//   //     godot_print!("Test result: {:?}", self.run_test(&test_name));
//   //   }
//   // }

//   pub fn run_test(&mut self, test_name: &str) -> bool
//   {
//     self.assertion_failed = false;
//     self.base.call(StringName::from(test_name), &[]);
//     self.assertion_failed
//   }

//   #[func]
//   fn assert_that(&mut self, what: bool, message: GString) -> bool {

//     if what {
//       return true;
//     }

//     self.assertion_failed = true;

//     godot_error!("GDScript assertion failed: {}", message);

//     false
//   }

//   #[func]
//   fn assert_fail(&mut self, message: GString) {
//     self.assertion_failed = true;
//   }
// }
