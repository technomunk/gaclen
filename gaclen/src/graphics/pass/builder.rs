use vulkano::format::{Format, PossibleDepthFormatDesc};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineCreationError};
use vulkano::pipeline::depth_stencil::{Compare, DepthStencil};
use vulkano::pipeline::shader::{SpecializationConstants, GraphicsEntryPointAbstract};
use vulkano::pipeline::raster::{CullMode, FrontFace, PolygonMode, Rasterization};
use vulkano::pipeline::vertex::{SingleBufferDefinition, VertexDefinition};
use vulkano::framebuffer::{AttachmentDescription, RenderPassDesc, RenderPassCreationError, Subpass};
use vulkano::image::ImageLayout;

use crate::graphics;
use graphics::device::Device;
use graphics::pass::graphical_pass;
use graphical_pass::{GraphicalPass, GraphicalRenderPassDescription};

use std::sync::Arc;

pub use vulkano::pipeline::input_assembly::PrimitiveTopology;
pub use vulkano::framebuffer::{StoreOp, LoadOp};

/// A structure for initializing [GraphicalPasses](struct.GraphicalPass).
pub struct GraphicalPassBuilder<VI, VS, VSS, FS, FSS> {
	vertex_input: VI,
	vertex_shader: (VS, VSS),
	primitive_topology: PrimitiveTopology,
	rasterization: Rasterization,
	fragment_shader: (FS, FSS),
	depth_stencil: DepthStencil,

	samples: u32,
	attachments: Vec<AttachmentDescription>,
	depth_attachment: Option<usize>,
}

/// Error during GraphicalPassBuilder setup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentError {
	/// The format supplied cannot be used for requested purposes.
	InvalidFormat,
	/// A depth attachment already exists while trying to add one.
	/// 
	/// Contains the index of existing attachment.
	DepthAttachmentAlreadyExists(usize),
}

/// Error during GraphicalPassBuilder::build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
	/// Error during creation of a [RenderPass].
	RenderPassCreation(RenderPassCreationError),
	/// Error during creation of a [GraphicsPipeline].
	GraphicsPipelineCreation(GraphicsPipelineCreationError),
	/// No attachments were added to the pass, therefore no invocation is possible!
	NoAttachments,
}

impl GraphicalPassBuilder<(), (), (), (), ()> {
	pub(super) fn new() -> Self {
		Self {
			vertex_input: (),
			vertex_shader: ((), ()),
			primitive_topology: PrimitiveTopology::TriangleList,
			rasterization: Rasterization::default(),
			fragment_shader: ((), ()),
			depth_stencil: DepthStencil::default(),

			samples: 1,
			attachments: Vec::default(),
			depth_attachment: None,
		}
	}
}

impl<VI, VS, VSS, FS, FSS> GraphicalPassBuilder<VI, VS, VSS, FS, FSS> {
	/// Use provided vertex input type.
	pub fn vertex_input<T>(self, vertex_input: T) -> GraphicalPassBuilder<T, VS, VSS, FS, FSS> {
		GraphicalPassBuilder {
			vertex_input: vertex_input,
			vertex_shader: self.vertex_shader,
			primitive_topology: self.primitive_topology,
			rasterization: self.rasterization,
			fragment_shader: self.fragment_shader,
			depth_stencil: self.depth_stencil,

			samples: self.samples,
			attachments: self.attachments,
			depth_attachment: self.depth_attachment,
		}
	}

	/// Use a single buffer of provided vertex type as input.
	pub fn single_buffer_input<V>(self) -> GraphicalPassBuilder<SingleBufferDefinition<V>, VS, VSS, FS, FSS> { self.vertex_input(SingleBufferDefinition::<V>::new()) }

	/// Use given [PrimitiveTopology].
	/// 
	/// Default is [PrimitiveTopology::TriangleList].
	pub fn primitive_topology(self, topology: PrimitiveTopology) -> Self { Self { primitive_topology: topology, .. self } }
	/// Use [PrimitiveTopology::PointList]. 
	pub fn point_list(self) -> Self { self.primitive_topology(PrimitiveTopology::PointList) }
	/// Use [PrimitiveTopology::LineList].
	pub fn line_list(self) -> Self { self.primitive_topology(PrimitiveTopology::LineList) }
	/// Use [PrimitiveTopology::LineStrip].
	pub fn line_strip(self) -> Self { self.primitive_topology(PrimitiveTopology::LineStrip) }
	/// Use [PrimitiveTopology::TriangleList].
	/// 
	/// This is the default.
	pub fn triangle_list(self) -> Self { self.primitive_topology(PrimitiveTopology::TriangleList) }
	/// Use [PrimitiveTopology::TriangleStrip].
	pub fn triangle_strip(self) -> Self { self.primitive_topology(PrimitiveTopology::TriangleStrip) }
	/// Use [PrimitiveTopology::TriangleFan].
	pub fn triangle_fan(self) -> Self { self.primitive_topology(PrimitiveTopology::TriangleFan) }
	/// Use [PrimitiveTopology::LineListWithAdjacency].
	pub fn line_list_with_adjacency(self) -> Self { self.primitive_topology(PrimitiveTopology::LineListWithAdjacency) }	
	/// Use [PrimitiveTopology::LineStripWithAdjacency].
	pub fn line_strip_with_adjacency(self) -> Self { self.primitive_topology(PrimitiveTopology::LineStripWithAdjacency) }
	/// Use [PrimitiveTopology::TriangleListWithAdjacency].
	pub fn triangle_list_with_adjacency(self) -> Self { self.primitive_topology(PrimitiveTopology::TriangleListWithAdjacency) }
	/// Use [PrimitiveTopology::TriangleStripWithAdjacency].
	pub fn triangle_strip_with_adjacency(self) -> Self { self.primitive_topology(PrimitiveTopology::TriangleStripWithAdjacency) }
	/// Use [PrimitiveTopology::PatchList].
	pub fn patch_list(self, vertices_per_patch: u32) -> Self { self.primitive_topology(PrimitiveTopology::PatchList{ vertices_per_patch }) }

	/// Set whether to clamp depth values of vertices.
	/// 
	/// If true vertices with depth outside [0 : 1] range will be clamp to those values.
	/// If false those vertices will be dropped.
	pub fn clamp_depth(mut self, clamp: bool) -> Self { self.rasterization.depth_clamp = clamp; self }

	/// Use provided [PolygonMode] for rasterizer (disassemble input primitives into provided types).
	pub fn raster_polygon_mode(mut self, mode: PolygonMode) -> Self { self.rasterization.polygon_mode = mode; self }

	/// Use provided [CullMode] for rasterizer. Culled faces are dropped before fragment stage.
	/// 
	/// Default is [CullMode::None].
	pub fn cull_mode(mut self, mode: CullMode) -> Self { self.rasterization.cull_mode = mode; self }
	/// Don't cull (default).
	pub fn cull_none(self) -> Self { self.cull_mode(CullMode::None) }
	/// Cull front faces.
	pub fn cull_front(self) -> Self { self.cull_mode(CullMode::Front) }
	/// Cull back faces.
	pub fn cull_back(self) -> Self { self.cull_mode(CullMode::Back) }
	/// Cull both back and front faces.
	pub fn cull_front_and_back(self) -> Self { self.cull_mode(CullMode::FrontAndBack) }

	/// Use provided [FrontFace].
	/// 
	/// Default is [FrontFace::CounterClockwise].
	pub fn front_face(mut self, face: FrontFace) -> Self { self.rasterization.front_face = face; self }
	/// Set clockwise faces as front.
	pub fn front_face_clockwise(self) -> Self { self.front_face(FrontFace::Clockwise) }
	/// Set counter-clockwise faces as front.
	/// 
	/// This is the default.
	pub fn front_face_counter_clockwise(self) -> Self { self.front_face(FrontFace::CounterClockwise) }

	/// Set the width of the lines drawn in pixels.
	pub fn line_width(mut self, width: f32) -> Self { self.rasterization.line_width = Some(width); self }

	// TODO: support this
	// /// Set the width of the lines drawn as dynamic, requiring their specification during draw call.
	// pub fn line_width_dynamic(mut self) -> Self { self.rasterization.line_width = None; self }

	/// Set whether to write to the depth buffer.
	/// 
	/// Default is `false`.
	pub fn depth_write(mut self, write: bool) -> Self { self.depth_stencil.depth_write = write; self }

	/// Set the operation to use for the depth test.
	/// 
	/// Default is `always`.
	pub fn depth_test_op(mut self, operation: Compare) -> Self { self.depth_stencil.depth_compare = operation; self }
	/// Set the depth test to always fail.
	pub fn depth_test_never(self) -> Self { self.depth_test_op(Compare::Never) }
	/// Set the depth test to pass if `value < reference_value`.
	pub fn depth_test_less(self) -> Self { self.depth_test_op(Compare::Less) }
	/// Set the depth test to pass if `value == reference_value`.
	pub fn depth_test_equal(self) -> Self { self.depth_test_op(Compare::Equal) }
	/// Set the depth test to pass if `value <= reference_value`.
	pub fn depth_test_less_or_equal(self) -> Self  { self.depth_test_op(Compare::LessOrEqual) }
	/// Set the depth test to pass if `value > reference_value`.
	pub fn depth_test_greater(self) -> Self { self.depth_test_op(Compare::Greater) }
	/// Set the depth test to pass if `value != reference_value`.	
	pub fn depth_test_not_equal(self) -> Self { self.depth_test_op(Compare::NotEqual) }
	/// Set the depth test to pass if `value >= reference_value`.	
	pub fn depth_test_greater_or_equal(self) -> Self { self.depth_test_op(Compare::GreaterOrEqual) }
	/// Set the depth test to always pass (default).
	pub fn depth_test_always(self) -> Self { self.depth_test_op(Compare::Always) }

	/// Use basic forward depth test.
	/// 
	/// Shortcut for `depth_write(true)` and `depth_test_less()`.
	pub fn basic_depth_test(self) -> Self { self.depth_write(true).depth_test_less() }
	/// Use basic inverse depth test.
	/// 
	/// Should be used with inverse depth buffer.
	/// Shortcut for `depth_write(true)` and `depth_test_greater()`.
	pub fn inverse_depth_test(self) -> Self { self.depth_write(true).depth_test_greater() }

	/// Use given vertex shader with given specialization constants.
	pub fn vertex_shader<S, SC>(self, shader: S, specialization: SC)
	-> GraphicalPassBuilder<VI, S, SC, FS, FSS> 
	where
		S : GraphicsEntryPointAbstract<SpecializationConstants = SC>,
		SC : SpecializationConstants,
	{
		GraphicalPassBuilder {
			vertex_input: self.vertex_input,
			vertex_shader: (shader, specialization),
			primitive_topology: self.primitive_topology,
			rasterization: self.rasterization,
			fragment_shader: self.fragment_shader,
			depth_stencil: self.depth_stencil,

			samples: self.samples,
			attachments: self.attachments,
			depth_attachment: self.depth_attachment,
		}
	}

	/// Use given fragment shader with given specialization constants.
	pub fn fragment_shader<S, SC>(self, shader: S, specialization: SC)
	-> GraphicalPassBuilder<VI, VS, VSS, S, SC>
	where
		S : GraphicsEntryPointAbstract<SpecializationConstants = SC>,
		SC : SpecializationConstants,
	{
		GraphicalPassBuilder {
			vertex_input: self.vertex_input,
			vertex_shader: self.vertex_shader,
			primitive_topology: self.primitive_topology,
			rasterization: self.rasterization,
			fragment_shader: (shader, specialization),
			depth_stencil: self.depth_stencil,

			samples: self.samples,
			attachments: self.attachments,
			depth_attachment: self.depth_attachment,
		}
	}

	/// Append an image attachment (resource that is drawn to) to this pass.
	pub fn add_image_attachment(mut self, format: Format, load: LoadOp, store: StoreOp) -> Self {
		self.attachments.push(AttachmentDescription{
			format,
			samples: self.samples,
			load,
			store,
			stencil_load: LoadOp::DontCare,
			stencil_store: StoreOp::DontCare,
			initial_layout: ImageLayout::ColorAttachmentOptimal,
			final_layout: ImageLayout::ColorAttachmentOptimal,
		});
		self
	}

	/// Append an image attachment (resource that is drawn to) to this pass.
	/// 
	/// In particular set up the pass to use swapchain image (frame result) of a device.
	pub fn add_image_attachment_swapchain(self, device: &Device, load: LoadOp) -> Self {
		self.add_image_attachment(device.swapchain.format(), load, StoreOp::Store)
	}

	/// Append an image attachment (resource that is drawn to) to this pass.
	/// 
	/// Shorthand for supplying LoadOp::Clear to add_image_attachment_swapchain.
	pub fn add_image_attachment_swapchain_cleared(self, device: &Device) -> Self {
		self.add_image_attachment_swapchain(device, LoadOp::Clear)
	}

	/// Append a depth-buffer attachment (resource that is drawn to) to this pass.
	/// 
	/// May fail if a depth attachment was already appended (currently only 1 is supported at a time).
	pub fn add_depth_attachment(mut self, format: Format, load: LoadOp, store: StoreOp) -> Result<Self, AttachmentError> {
		if format.is_depth() {
			match self.depth_attachment {
				Some(index) => Err(AttachmentError::DepthAttachmentAlreadyExists(index)),
				None => {
					self.depth_attachment = Some(self.attachments.len());
					self.attachments.push(AttachmentDescription{
						format: format,
						samples: self.samples,
						load: load,
						store: store,
						stencil_load: load,
						stencil_store: store,
						initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
						final_layout: ImageLayout::DepthStencilAttachmentOptimal,
					});
					Ok(self)
				}
			}
		} else {
			Err(AttachmentError::InvalidFormat)
		}
	}

	/// Append a depth-buffer attachment (resource that is drawn to) to this pass.
	/// 
	/// May fail if a depth attachment was already appended (currently only 1 is supported at a time).
	/// In particular set up the pass to use swapchain depth of a device.
	pub fn add_depth_attachment_swapchain(self, device: &Device, load: LoadOp, store: StoreOp) -> Result<Self, AttachmentError> {
		self.add_depth_attachment(device.swapchain_depth_format, load, store)
	}

	/// Append a depth-buffer attachment (resource that is drawn to) to this pass.
	/// 
	/// Shorthand for supplying StoreOp::DontCare as store parameter to add_depth_attachment_swapchain.
	pub fn add_depth_attachment_swapchain_discard(self, device: &Device, load: LoadOp) -> Result<Self, AttachmentError> {
		self.add_depth_attachment_swapchain(device, load, StoreOp::DontCare)
	}

	/// Append a depth-buffer attachment (resource that is drawn to) to this pass.
	/// 
	/// Shorthand for supplying StoreOp::Store as store parameter to add_depth_attachment_swapchain.
	pub fn add_depth_attachment_swapchain_preserve(self, device: &Device, load: LoadOp) -> Result<Self, AttachmentError> {
		self.add_depth_attachment_swapchain(device, load, StoreOp::Store)
	}
}

impl<VI, VS, VSS, FS, FSS> GraphicalPassBuilder<VI, VS, VSS, FS, FSS>
where
	VS : GraphicsEntryPointAbstract<SpecializationConstants=VSS>,
	FS : GraphicsEntryPointAbstract<SpecializationConstants=FSS>,
	VSS : SpecializationConstants,
	FSS : SpecializationConstants,
	VS::PipelineLayout : Send + Sync + Clone + 'static,
	FS::PipelineLayout : Send + Sync + Clone + 'static,
	VI : VertexDefinition<VS::InputDefinition> + Send + Sync + 'static,
{
	pub fn build(self, device: &Device)
	-> Result<GraphicalPass<dyn GraphicsPipelineAbstract + Send + Sync>, BuildError> {
		if self.attachments.is_empty() {
			return Err(BuildError::NoAttachments)
		};

		let render_pass = {
			let description = GraphicalRenderPassDescription {
				attachments: self.attachments,
				depth_attachment: self.depth_attachment,
			};
			Arc::new(description.build_render_pass(device.device.clone())?)
		};

		let pipeline = {
			let builder = GraphicsPipeline::start()
			.vertex_input(self.vertex_input)
			.vertex_shader(self.vertex_shader.0, self.vertex_shader.1)
			.primitive_topology(self.primitive_topology)
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(self.fragment_shader.0, self.fragment_shader.1)
			.depth_stencil(self.depth_stencil)
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.depth_clamp(self.rasterization.depth_clamp)
			;

			let builder = match self.rasterization.polygon_mode {
				PolygonMode::Point => builder.polygon_mode_point(),
				PolygonMode::Line => builder.polygon_mode_line(),
				PolygonMode::Fill => builder.polygon_mode_fill(),
			};

			let builder = match self.rasterization.cull_mode {
				CullMode::None => builder.cull_mode_disabled(),
				CullMode::Front => builder.cull_mode_front(),
				CullMode::Back => builder.cull_mode_back(),
				CullMode::FrontAndBack => builder.cull_mode_front_and_back(),
			};

			let builder = match self.rasterization.front_face {
				FrontFace::Clockwise => builder.front_face_clockwise(),
				FrontFace::CounterClockwise => builder.front_face_counter_clockwise(),
			};

			let builder = match self.rasterization.line_width {
				Some(width) => builder.line_width(width),
				None => builder,
			};

			Arc::new(builder.build(device.device.clone())?)
		};
		
		Ok(GraphicalPass { render_pass, pipeline, })
	}
}

impl From<RenderPassCreationError> for BuildError {
	fn from(err: RenderPassCreationError) -> Self { Self::RenderPassCreation(err) }
}
impl From<GraphicsPipelineCreationError> for BuildError {
	fn from(err: GraphicsPipelineCreationError) -> Self { Self::GraphicsPipelineCreation(err) }
}
