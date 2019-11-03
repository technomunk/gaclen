//! Infrastructure for interpreting and computing data.
//! 
//! Example passes are:
//! - **Shadow** - drawing a scene from the point of view of a light source in order to save depth information.
//! - **Albedo** - drawing typically-represented geometry with lighting and optional shading.
//! - **Post-process** - screen-space based techniques for processing image before presenting it on the screen.

use super::device::Device;
use super::ResizeError;

use vulkano::image::{AttachmentImage, ImageCreationError, ImageUsage, ImageViewAccess};
use vulkano::format::Format;
use vulkano::framebuffer::{FramebufferCreationError, RenderPassAbstract, RenderPassCreationError, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineCreationError};
use vulkano::pipeline::shader::{GraphicsEntryPointAbstract};

use std::sync::Arc;

/// A GraphicalPass produces some images as its result.
pub trait GraphicalPass {
	type RenderPass: ?Sized + RenderPassAbstract + Send + Sync + 'static;
	type Pipeline: ?Sized + GraphicsPipelineAbstract + Send + Sync + 'static;

	/// Get the underlying vulkano render pass of the GraphicalPass.
	fn render_pass(&self) -> Arc<Self::RenderPass>;
	/// Get the underlying pipeline of the GraphicalPass.
	fn pipeline(&self) -> Arc<Self::Pipeline>;
	/// Get a vector of image-accesses that the pass will write to.
	/// Note that for present images (one that are presented on the screen) are not owned by the pass and therefore are skipped here.
	/// In order to have access to present images implement PresentPass for a given GraphicalPass.
	/// PresentPasses get the final screen image in the first index.
	fn images(&self) -> Vec<&dyn ImageViewAccess>;
}

/// A PresentPass produces images that are presentable to the screen.
/// As a result such a pass does not need to supply its own images. 
pub trait PresentPass : GraphicalPass {}

/// A DependentPass is one that uses results of another pass as input.
pub trait DependentPass<P: GraphicalPass> : GraphicalPass {}

/// Error during creation of the AlbedoPass.
#[derive(Debug)]
pub enum PassCreationError {
	/// Error during creation of the underlying vulkan render-pass.
	RenderPass(RenderPassCreationError),
	/// Error during creation of the underlying vulkan graphics-pipeline.
	GraphicsPipeline(GraphicsPipelineCreationError),
	/// Error during initial resizing.
	Resize(ResizeError),
	/// Error trying to create attached image resource.
	Image(ImageCreationError),
	/// Error trying to create framebuffers for the pass images.
	Framebuffer(FramebufferCreationError),
	/// The custom format supplied for the pass is not supported for that pass type.
	IncorrectFormat,
}

/// Shadow pass renders geometry to a depth buffer.
/// 
/// The depth buffer can then be used to calculate whether a fragment is within view of the light source or not.
pub struct ShadowPass {
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

	image: Arc<AttachmentImage<Format>>,
}

pub struct AlbedoPass {
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl GraphicalPass for ShadowPass {
	type RenderPass = dyn RenderPassAbstract + Send + Sync + 'static;
	type Pipeline = dyn GraphicsPipelineAbstract + Send + Sync + 'static;

	#[inline(always)]
	fn render_pass(&self) -> Arc<Self::RenderPass> { self.render_pass.clone() }
	#[inline(always)]
	fn pipeline(&self) -> Arc<Self::Pipeline> { self.graphics_pipeline.clone() }
	#[inline(always)]
	fn images(&self) -> Vec<&dyn ImageViewAccess> { vec![&self.image] }
}

impl GraphicalPass for AlbedoPass {
	type RenderPass = dyn RenderPassAbstract + Send + Sync + 'static;
	type Pipeline = dyn GraphicsPipelineAbstract + Send + Sync + 'static;

	#[inline(always)]
	fn render_pass(&self) -> Arc<Self::RenderPass> { self.render_pass.clone() }
	#[inline(always)]
	fn pipeline(&self) -> Arc<Self::Pipeline> { self.graphics_pipeline.clone() }
	#[inline(always)]
	fn images(&self) -> Vec<&dyn ImageViewAccess> { Vec::new() }
}
impl PresentPass for AlbedoPass {}

impl ShadowPass {
	/// Create a new ShadowPass.
	/// 
	/// Create a new ShadowPass using provided shader instances, specialization constants, depth framebuffer format and dimensions.
	/// 
	/// Template parameters:
	/// - `VS` : vertex shader to be used in the pass.
	/// - `FS` : fragment shader to be used in the pass.
	/// - `T`  : vertex data type to be used in the pass.
	pub fn new<VS, FS, T>(
		device: &Device,
		vertex_shader: VS,
		vssc: VS::SpecializationConstants,
		fragment_shader: FS,
		fssc: FS::SpecializationConstants,
		dimensions: [u32; 2],
		format: Format
	) -> Result<Self, PassCreationError>
	where
		VS : GraphicsEntryPointAbstract,
		FS : GraphicsEntryPointAbstract,
		VS::PipelineLayout : Send + Sync + Clone + 'static,
		FS::PipelineLayout : Send + Sync + Clone + 'static,
		T : Send + Sync + 'static,
		vulkano::pipeline::vertex::SingleBufferDefinition<T> : vulkano::pipeline::vertex::VertexDefinition<VS::InputDefinition>
	{
		if format.ty().is_depth_and_or_stencil() { return Err(PassCreationError::IncorrectFormat) };

		let render_pass = Arc::new(vulkano::single_pass_renderpass!(
			device.device.clone(),
			attachments: {
				depth: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
				}
			},
			pass: {
				color: [],
				depth_stencil: {depth}
			})?);

		let graphics_pipeline = Arc::new(GraphicsPipeline::start()
			.vertex_input_single_buffer::<T>()
			.vertex_shader(vertex_shader, vssc)
			.triangle_list()
			.cull_mode_back()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fragment_shader, fssc)
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.device.clone())?);
		
		let usage = ImageUsage {
			sampled: true,
			storage: true,
			depth_stencil_attachment: true,
			.. ImageUsage::none()
		};
		
		let image = AttachmentImage::with_usage(device.device.clone(), dimensions, format, usage)?;

		let pass = Self {
			graphics_pipeline,
			render_pass,
			image,
		};
		
		Ok(pass)
	}
}

impl AlbedoPass {
	/// Create a new AlbedoPass.
	pub fn new<VS, FS, T>(
		device: &Device,
		vertex_shader: VS,
		vssc: VS::SpecializationConstants,
		fragment_shader: FS,
		fssc: FS::SpecializationConstants
	) -> Result<Self, PassCreationError>
	where
		VS : GraphicsEntryPointAbstract,
		FS : GraphicsEntryPointAbstract,
		VS::PipelineLayout : Send + Sync + Clone + 'static,
		FS::PipelineLayout : Send + Sync + Clone + 'static,
		T : Send + Sync + 'static,
		vulkano::pipeline::vertex::SingleBufferDefinition<T> : vulkano::pipeline::vertex::VertexDefinition<VS::InputDefinition>
	{
		let render_pass = Arc::new(vulkano::single_pass_renderpass!(
			device.device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: device.swapchain.format(),
					samples: 1,
				},
				depth: {
					load: Clear,
					store: DontCare,
					format: Format::D16Unorm,
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {depth}
			})?);

		let graphics_pipeline = Arc::new(GraphicsPipeline::start()
			.vertex_input_single_buffer::<T>()
			.vertex_shader(vertex_shader, vssc)
			.triangle_list()
			.cull_mode_back()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fragment_shader, fssc)
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.device.clone())?);
		
		let pass = Self {
			graphics_pipeline,
			render_pass,
		};
		Ok(pass)
	}
}

impl From<RenderPassCreationError> for PassCreationError {
	fn from(err: RenderPassCreationError) -> PassCreationError { PassCreationError::RenderPass(err) }
}
impl From<GraphicsPipelineCreationError> for PassCreationError {
	fn from(err: GraphicsPipelineCreationError) -> PassCreationError { PassCreationError::GraphicsPipeline(err) }
}
impl From<ResizeError> for PassCreationError {
	fn from(err: ResizeError) -> PassCreationError { PassCreationError::Resize(err) }
}
impl From<ImageCreationError> for PassCreationError {
	fn from(err: ImageCreationError) -> PassCreationError { PassCreationError::Image(err) }
}
impl From<FramebufferCreationError> for PassCreationError {
	fn from(err: FramebufferCreationError) -> PassCreationError { PassCreationError::Framebuffer(err) }
}