use gd_rehearse::bench::*;
use godot::engine::Node;
use godot::engine::Object;
use godot::obj::Gd;
use godot::obj::InstanceId;
use godot::obj::NewAlloc;

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
fn bench_with_ctx(ctx: &BenchContext) -> InstanceId {
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

fn setup_function(ctx: &mut BenchContext) {
    let mut node = Node::new_alloc();
    let mut child = Node::new_alloc();
    child.set_name("SetupChild".into());
    node.add_child(child);

    ctx.setup_add_node(node, "SetupTest");
}

fn cleanup_function(ctx: &mut BenchContext) {
    ctx.remove_added_node("SetupTest");
}

#[gdbench(setup=setup_function)]
fn with_setup(ctx: &BenchContext) -> bool {
    let _setup = ctx.get_setup_node("SetupTest");
    let _child = ctx.get_node("SetupTest/SetupChild");
    true
}

#[gdbench(setup=setup_function, cleanup=cleanup_function)]
fn with_setup_and_cleanup(ctx: &BenchContext) -> bool {
    let _setup = ctx.get_setup_node("SetupTest");
    true
}
