use super::builder::GraphicalPassBuilder;

use std::sync::Arc;

/// Special marker for present passes.
/// PresentPasses use swapchain buffers (draw visible results on the screen).
pub struct PresentPass();

/// A GraphicalPass defines the device configuration used to execute draw commands.
/// 
/// There are 2 types of GraphicalPasses:
/// - **Internal** - the results of which are used by later passes, for example: shadow passes.
/// - **Present** - the results of which are visible on the screen, for example: final post-process, simple albedo.
pub struct GraphicalPass<P : ?Sized, RP : ?Sized, PP> {
	pub(in crate::graphics) pipeline: Arc<P>,
	pub(in crate::graphics) render_pass: Arc<RP>,

	pub(in crate::graphics) phantom: std::marker::PhantomData<PP>
}

impl GraphicalPass<(), (), ()> {
	pub fn start() -> GraphicalPassBuilder<(), (), (), (), ()> { GraphicalPassBuilder::new() }
}