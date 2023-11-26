/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use godot::engine::Object;
use godot::obj::Gd;
use godot_test::itest::*;
use godot_test::CaseContext;

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
fn test_with_ctx(ctx: &CaseContext) {
    let gd: Gd<Object> = ctx.scene_tree.clone().upcast();
    gd.instance_id();
}

#[gditest(keyword = "my new class")]
fn keyword_class_test() {}

#[gditest]
fn filter_me() {}

#[gditest]
fn filter_me_too() {}
