use super::builder::GraphicalPassBuilder;

use vulkano::framebuffer::{FramebufferCreationError, RenderPassAbstract, RenderPassCreationError, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineCreationError};

use std::sync::Arc;

/// Special marker for present passes.
/// PresentPasses use swapchain buffers (draw visible results on the screen).
pub struct PresentPass();

/// A GraphicalPass produces some images as its result.
pub struct GraphicalPass<P : ?Sized, RP : ?Sized, I, PP> {
	pub(in crate::graphics) pipeline: Arc<P>,
	pub(in crate::graphics) render_pass: Arc<RP>,
	pub(in crate::graphics) images: I,

	pub(in crate::graphics) phantom: std::marker::PhantomData<PP>
}

impl GraphicalPass<(), (), (), ()> {
	pub fn start() -> GraphicalPassBuilder<(), (), (), (), ()> { GraphicalPassBuilder::new() }
}