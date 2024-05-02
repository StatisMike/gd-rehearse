/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use gd_rehearse::itest::*;
use godot::engine::Object;
use godot::obj::Gd;

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
fn skipped_test() {
    let test = 1 + 1;
    assert_eq!(test, 1);
}

#[gditest(keyword = "with ctx")]
fn test_with_ctx(ctx: &TestContext) {
    let gd: Gd<Object> = ctx.scene_tree().clone().upcast();
    gd.instance_id();
}

#[gditest(keyword = "my new class")]
fn keyword_class_test() {}

#[gditest]
fn filter_me() {}

#[gditest]
fn filter_me_too() {}

#[gditest(scene_path = "res://with_path.tscn")]
fn with_test_path(ctx: &TestContext) {
    assert_eq!(
        ctx.scene_tree().get_scene_file_path().to_string(),
        "res://with_path.tscn"
    )
}

#[gditest(scene_path = "res://nonexistent.tscn")]
fn shouldnt_run_path() {
    let test = 1 + 1;
    assert_eq!(test, 1);
}

#[gditest(scene_path = "res://with_path.tscn")]
fn access_ctx_with_path(ctx: &TestContext) {
    let some_node = ctx.scene_tree().get_node_or_null("SomeNode".into());
    assert!(some_node.is_some());

    let value = some_node.unwrap().get("my_value".into());
    assert!(!value.is_nil());
    let val_as_int = value.to::<i32>();
    assert_eq!(val_as_int, 344);
}
