/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::panic::RefUnwindSafe;
use std::time::{Duration, Instant};

use godot::builtin::{GString, NodePath};
use godot::classes::Node;
use godot::meta::AsArg;
use godot::obj::{Gd, Inherits};

use crate::runner::panic::{unwind_result, UnwindError, UnwindResult};

use super::{Case, CaseContext};

/// Rust benchmark.
///
/// Created by using `#[gdbench]` macro and registered to run by test runner.
#[derive(Copy, Clone)]
pub struct RustBenchmark {
    pub name: &'static str,
    pub file: &'static str,
    pub skipped: bool,
    pub focused: bool,
    pub keyword: Option<&'static str>,
    pub scene_path: Option<&'static str>,
    #[allow(dead_code)]
    pub line: u32,
    pub function: fn(&BenchContext),
    pub setup_function: Option<fn(&mut BenchContext)>,
    pub cleanup_function: Option<fn(&mut BenchContext)>,
    pub repetitions: usize,
}

impl Case for RustBenchmark {
    fn get_case_name(&self) -> &str {
        self.name
    }
    fn get_case_file(&self) -> &str {
        self.file
    }
    fn is_case_focus(&self) -> bool {
        self.focused
    }
    fn is_case_skip(&self) -> bool {
        self.skipped
    }
    fn get_case_keyword(&self) -> &Option<&str> {
        &self.keyword
    }
    fn get_case_scene_path(&self) -> &Option<&str> {
        &self.scene_path
    }
    fn get_case_line(&self) -> u32 {
        self.line
    }
}

impl RustBenchmark {
    pub(crate) fn execute_setup_function(
        &self,
        ctx: BenchContext,
    ) -> Result<BenchContext, UnwindError> {
        if let Some(setup) = self.setup_function {
            let mut cloned_ctx = ctx.clone();

            let res: UnwindResult<BenchContext> = std::panic::catch_unwind(move || {
                (setup)(&mut cloned_ctx);
                Ok(cloned_ctx)
            });

            return unwind_result(res);
        }
        Ok(ctx)
    }

    pub(crate) fn execute_cleanup_function(
        &self,
        mut ctx: BenchContext,
    ) -> Result<BenchContext, CleanupError> {
        if self.setup_function.is_none() {
            return Ok(ctx);
        }
        if let Some(cleanup) = self.cleanup_function {
            let mut cloned_ctx = ctx.clone();

            let res: UnwindResult<BenchContext> = std::panic::catch_unwind(move || {
                (cleanup)(&mut cloned_ctx);
                Ok(cloned_ctx)
            });

            let res = unwind_result(res);
            if let Err(err) = res {
                return Err(CleanupError {
                    not_cleaned: false,
                    cause: Some(err),
                });
            }
            if let Ok(ctx) = res {
                if !ctx.added_nodes.is_empty() {
                    return Err(CleanupError {
                        not_cleaned: true,
                        cause: None,
                    });
                }
                return Ok(ctx);
            }
        }
        ctx.remove_all_added_nodes();
        Ok(ctx)
    }
}

/// Context for Rust Benchmarking case.
///
/// Object allowing access to the scene tree during `#[gdbench]` benchmarking function execution. Provides simple, general access to the
/// scene tree relative to [GdTestRunner](crate::runner::GdTestRunner) in which the banchmark takes place, as well as some utility functions
/// for retrieving nodes present in the scene or set up before benchmark with setup function.
///
/// Usage of specialized node-retrieving functions is recommended - some overhead will be still recorded, but the duration which is crucial
/// in benchmarking will be adjusted mostly.
///
/// ## Examples
///
/// ```no_run
/// use godot::prelude::*;
/// use gd_rehearse::bench::*;
///
/// fn setup_function(ctx: &mut BenchContext) {
///    let node = Node::new_alloc();
///    ctx.setup_add_node(node, "OnSetup");
/// }
///
/// #[gdbench(setup=setup_function, scene_path="res://scene_with_specific.tscn")]
/// fn with_setup(ctx: &BenchContext) -> bool {
///    let _from_setup = ctx.get_setup_node("OnSetup");
///    let _from_scene = ctx.get_node("SceneSpecific");
///    true
/// }
/// ```
#[derive(Clone)]
pub struct BenchContext {
    pub(crate) scene_tree: Gd<Node>,
    added_nodes: HashSet<GString>,
    sub_durations: RefCell<Duration>,
}

impl CaseContext for BenchContext {
    fn scene_tree(&self) -> &Gd<Node> {
        &self.scene_tree
    }

    fn get_node(&self, path: impl AsArg<NodePath>) -> Gd<Node> {
        let start = Instant::now();
        let out = self
            .scene_tree()
            .get_node_or_null(path)
            .expect("cannot get node");

        *self.sub_durations.borrow_mut() += start.elapsed();
        out
    }

    fn get_node_as<T: Inherits<Node>>(&self, path: impl AsArg<NodePath>) -> Gd<T> {
        let start = Instant::now();
        let out = self
            .scene_tree()
            .try_get_node_as(path)
            .expect("cannot get node as");

        *self.sub_durations.borrow_mut() += start.elapsed();
        out
    }
}

impl RefUnwindSafe for BenchContext {}

impl BenchContext {
    pub(crate) fn new(scene_tree: Gd<Node>) -> Self {
        Self {
            scene_tree,
            added_nodes: HashSet::new(),
            sub_durations: RefCell::new(Duration::default()),
        }
    }

    /// Removes all nodes added during the setup procedure.
    ///
    /// This method is called during default cleanup procedure, and needs to be called also when implementing some custom cleanup is required.
    pub fn remove_all_added_nodes(&mut self) {
        for node_path in self.added_nodes.drain() {
            if let Some(mut node) = self.scene_tree.get_node_or_null(node_path.arg()) {
                node.queue_free()
            }
        }
    }

    /// Removes single node added during the setup procedure.
    ///
    /// For usage in custom cleanup procedure, if you need to remove the nodes in specific order. All nodes need to be cleaned up during cleanup.
    pub fn remove_added_node(&mut self, name: impl Into<GString>) {
        let name: GString = name.into();
        if let Some(mut node) = self.scene_tree.get_node_or_null(name.arg()) {
            node.queue_free();
            self.added_nodes.remove(&name);
        }
    }

    /// Add node to scene in which the benchmark will be processed.
    ///
    /// This method should be called only during setup procedure for a benchmark, to prepare some objects which shouldn't be generated
    /// within the benchmark run itself.
    pub fn setup_add_node(&mut self, node: Gd<Node>, name: impl Into<GString>) {
        let name = name.into();
        let mut node = node.clone();
        node.set_name(&name);
        self.scene_tree.add_child(&node);
        self.added_nodes.insert(name);
    }

    /// Gets node from current benchmark context that was set up.
    ///
    /// ## Panics
    ///
    /// If no node with `name` was set up during setup function.
    pub fn get_setup_node(&self, name: impl Into<GString>) -> Gd<Node> {
        let start = Instant::now();
        let gstring: GString = name.into();
        if !self.added_nodes.contains(&gstring) {
            panic!("no node with name: `{gstring}` were set up");
        }
        let out = self
            .scene_tree
            .get_node_or_null(gstring.arg())
            .expect("cannot get setup node");
        *self.sub_durations.borrow_mut() += start.elapsed();
        out
    }

    /// Gets node from current benchmark context that was set up, upcasted to `T`.
    ///
    /// ## Panics
    ///
    /// If no node with `name` was set up during setup function, or cannot be upcasted to `T`.
    pub fn get_setup_node_as<T: Inherits<Node>>(&self, name: impl Into<GString>) -> Gd<T> {
        let start = Instant::now();
        let gstring: GString = name.into();
        if !self.added_nodes.contains(&gstring) {
            panic!("no node with name: `{gstring}` were set up");
        }
        let out = self
            .scene_tree
            .try_get_node_as(gstring.arg())
            .expect("cannot get setup node as");
        *self.sub_durations.borrow_mut() += start.elapsed();
        out
    }

    /// Set inner duration adjustment to zero.
    pub(crate) fn zero_duration(&mut self) {
        self.sub_durations.replace(Duration::default());
    }

    /// Gets duration from `start` adjusted for operations made by methods implemented in `BenchContext`.
    pub(crate) fn get_adjusted_duration(&mut self, start: Instant) -> Duration {
        let mut duration = start.elapsed();
        let subtraction = self.sub_durations.replace(Duration::default());
        duration -= subtraction;

        duration
    }
}

#[derive(Debug)]
pub(crate) enum BenchError {
    Setup(UnwindError),
    Execution(UnwindError),
    Cleanup(CleanupError),
}

impl Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchError::Setup(err) => write!(f, "[setup] {err}"),
            BenchError::Execution(err) => write!(f, "[execution] {err}"),
            BenchError::Cleanup(err) => write!(f, "[cleanup] {err}"),
        }
    }
}

impl Error for BenchError {}

#[derive(Debug)]
pub(crate) struct CleanupError {
    not_cleaned: bool,
    cause: Option<UnwindError>,
}

impl Display for CleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.not_cleaned {
            return write!(f, "some setup nodes are still present. Call `BenchContext::remove_added_nodes()` in your cleanup function");
        }
        if let Some(cause) = &self.cause {
            write!(f, "{cause}")
        } else {
            write!(f, "panic during cleanup procedure")
        }
    }
}

impl Error for CleanupError {
    fn cause(&self) -> Option<&dyn Error> {
        if let Some(cause) = &self.cause {
            return Some(cause);
        }
        None
    }
}

#[doc(hidden)]
/// Signal to the compiler that a value is used (to avoid optimization).
pub fn bench_used<T: Sized>(value: T) {
    std::hint::black_box(value);
}
