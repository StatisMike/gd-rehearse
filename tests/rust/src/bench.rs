use gd_rehearse::bench::*;
use gd_rehearse::CaseContext;
use godot::engine::Object;
use godot::obj::Gd;
use godot::obj::InstanceId;

#[gdbench(focus)]
fn focused_bench() -> i32 {
    243
}

#[gdbench(skip)]
fn skipped_bench() -> i32 {
    234
}

#[gdbench]
fn normal_bench() -> i32 {
    324
}

#[gdbench(keyword = "with ctx")]
fn bench_with_ctx(ctx: &CaseContext) -> InstanceId {
    let gd: Gd<Object> = ctx.scene_tree().clone().upcast();
    gd.instance_id()
}

#[gdbench(scene_path = "res://with_path.tscn")]
fn path_bench() -> i32 {
    324
}

#[gdbench(scene_path = "res://nonexistent.tscn")]
fn shouldnt_run_path() -> i8 {
    let test = 1 + 1;
    assert_eq!(test, 1);
    test
}
