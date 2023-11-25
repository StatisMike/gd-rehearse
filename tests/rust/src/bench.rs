use godot::engine::Object;
use godot::obj::Gd;
use godot::obj::InstanceId;
use godot_test::gdbench;
use godot_test::CaseContext;

#[gdbench]
fn focused_bench() -> i32 {
    243
}

#[gdbench(skip)]
fn skipped_bench() -> i32 {
    234
}

#[gdbench(keyword = "with ctx")]
fn bench_with_ctx(ctx: &CaseContext) -> InstanceId {
    let gd: Gd<Object> = ctx.scene_tree.clone().upcast();
    gd.instance_id()
}
