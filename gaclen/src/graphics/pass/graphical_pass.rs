use super::builder::GraphicalPassBuilder;

use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet, PersistentDescriptorSetBuilder};
use vulkano::format::ClearValue;
use vulkano::framebuffer::{AttachmentDescription, PassDescription, RenderPass, RenderPassDesc, RenderPassDescClearValues, PassDependencyDescription};
use vulkano::image::ImageLayout;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;

/// A GraphicalPass defines the device configuration used to execute draw commands.
/// 
/// There are 2 types of GraphicalPasses:
/// - **Internal** - the results of which are used by later passes, for example: shadow passes.
/// - **Present** - the results of which are visible on the screen, for example: final post-process, simple albedo.
pub struct GraphicalPass<P: ?Sized, A : AttachmentCollection> {
	pub(in crate::graphics) pipeline: Arc<P>,
	pub(in crate::graphics) render_pass: Arc<RenderPass<GraphicalRenderPassDescription<A>>>,
}

impl GraphicalPass<(), ()> {
	/// Begin building a GraphicalPass.
	pub fn start() -> GraphicalPassBuilder<(), (), (), (), (), ()> { GraphicalPassBuilder::new() }
}

impl<P, A> GraphicalPass<P, A>
where
	P : GraphicsPipelineAbstract + Send + Sync + ?Sized,
	A : AttachmentCollection,
{
	/// Start building a new persistent descriptor set.
	pub fn start_persistent_descriptor_set(&self, index: usize) -> PersistentDescriptorSetBuilder<Arc<P>, ()> {
		PersistentDescriptorSet::start(self.pipeline.clone(), index)
	}
}

pub struct GraphicalRenderPassDescription<A : AttachmentCollection> {
	/// Image attachments of the render pass.
	pub attachments: A,
	/// Depth stencil attachment index.
	pub depth_attachment: Option<usize>,
}

type Attachment = (AttachmentType, AttachmentDescription);
pub trait AttachmentCollection {
	const ATTACHMENT_COUNT: usize;
	fn get(&self, num: usize) -> Option<Attachment>;
}

impl AttachmentCollection for () {
	const ATTACHMENT_COUNT: usize = 0;
	#[inline]
	fn get(&self, _: usize) -> Option<Attachment> { None }
}
impl AttachmentCollection for (AttachmentType, AttachmentDescription) {
	const ATTACHMENT_COUNT: usize = 1;
	#[inline]
	fn get(&self, num: usize) -> Option<Attachment> {
		match num {
			0 => Some(self.clone()),
			_ => None,
		}
	}
}
impl<A : AttachmentCollection> AttachmentCollection for (A, (AttachmentType, AttachmentDescription)) {
	const ATTACHMENT_COUNT: usize = A::ATTACHMENT_COUNT + 1;
	#[inline]
	fn get(&self, num: usize) -> Option<Attachment> {
		match num {
			0 => Some(self.1.clone()),
			num => self.0.get(num - 1),
		}
	}
}

impl Clone for Attachment { fn clone(&self) -> Self { (self.0, self.1) } }

pub enum AttachmentType {
	SwapchainColor,
	SwapchainDepth,
	General,
}

impl<A : AttachmentCollection> GraphicalRenderPassDescription<A> {
	#[inline]
	pub fn push_attachment(self, r#type: AttachmentType, desc: AttachmentDescription) -> GraphicalRenderPassDescription<(A, (AttachmentType, AttachmentDescription))> {
		GraphicalRenderPassDescription{ attachments: (self.attachments, (r#type, desc)), depth_attachment: self.depth_attachment }
	}
	#[inline]
	pub fn attachment_count() -> usize { A::ATTACHMENT_COUNT }
}
impl<A : AttachmentCollection> GraphicalRenderPassDescription<(A, (AttachmentType, AttachmentDescription))> {
	#[inline]
	pub fn pop_attachment(self) -> (GraphicalRenderPassDescription<A>, (AttachmentType, AttachmentDescription)) {
		let (remainder, popped) = self.attachments;
		(GraphicalRenderPassDescription{ attachments: remainder, depth_attachment: self.depth_attachment }, popped)
	}
}

unsafe impl<A : AttachmentCollection> RenderPassDesc for GraphicalRenderPassDescription<A> {
	#[inline]
	fn num_attachments(&self) -> usize { A::ATTACHMENT_COUNT }
	
	#[inline]
	fn attachment_desc(&self, num: usize) -> Option<AttachmentDescription> {
		match self.attachments.get(A::ATTACHMENT_COUNT - num - 1) {
			Some((_, desc)) => Some(desc),
			None => None,
		}
	}

	#[inline]
	fn num_subpasses(&self) -> usize { 1 }

	#[inline]
	fn subpass_desc(&self, num: usize) -> Option<PassDescription> {
		if num == 0 {
			let color_attachments = {
				if let Some(depth_index) = self.depth_attachment {
					let mut color_attachments = Vec::with_capacity(A::ATTACHMENT_COUNT - 1);
					for i in 0..depth_index {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal));
					}
					for i in depth_index + 1 .. A::ATTACHMENT_COUNT {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal));
					}
					color_attachments
				} else {
					let mut color_attachments = Vec::with_capacity(A::ATTACHMENT_COUNT);
					for i in 0..A::ATTACHMENT_COUNT {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal))
					};
					color_attachments
				}
			};
			let depth_stencil = match self.depth_attachment {
				Some(index) => Some((index, ImageLayout::DepthStencilAttachmentOptimal)),
				None => None,
			};
			Some(PassDescription{
				color_attachments,
				depth_stencil,
				input_attachments: Vec::default(),
				resolve_attachments: Vec::default(),
				preserve_attachments: Vec::default(),
			})
		} else {
			None
		}
	}

	fn num_dependencies(&self) -> usize { 0 }

	fn dependency_desc(&self, num: usize) -> Option<PassDependencyDescription> { None }
}

unsafe impl<A : AttachmentCollection> RenderPassDescClearValues<Vec<ClearValue>> for GraphicalRenderPassDescription<A> {
	// TODO/vulkano: find out what this is supposed to do.
	fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<dyn Iterator<Item = ClearValue>> { Box::new(values.into_iter()) }
}
