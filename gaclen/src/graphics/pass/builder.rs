use vulkano::pipeline::shader::SpecializationConstants;
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::pipeline::raster::Rasterization;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineCreationError};
use vulkano::pipeline::shader::GraphicsEntryPointAbstract;
use vulkano::pipeline::vertex::VertexDefinition;
use vulkano::framebuffer::{RenderPassAbstract, RenderPassCreationError, Subpass};

use crate::graphics;
use graphics::device::Device;
use graphics::pass::graphical_pass;
use graphical_pass::{GraphicalPass, PresentPass};

use std::sync::Arc;

/// A structure for initializing [GraphicalPasses](struct.GraphicalPass)
pub struct GraphicalPassBuilder<VI, VS, VSS, FS, FSS> {
	vertex_input: VI,
	vertex_shader: (VS, VSS),
	primitive_topology: PrimitiveTopology,
	rasterization: Rasterization,
	fragment_shader: (FS, FSS),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
	RenderPassCreation(RenderPassCreationError),
	GraphicsPipelineCreation(GraphicsPipelineCreationError),
}

impl GraphicalPassBuilder<(), (), (), (), ()> {
	pub(super) fn new() -> Self {
		Self {
			vertex_input: (),
			vertex_shader: ((), ()),
			primitive_topology: PrimitiveTopology::PointList,
			rasterization: Rasterization::default(),
			fragment_shader: ((), ()),
		}
	}
}

impl<VI, VS, VSS, FS, FSS> GraphicalPassBuilder<VI, VS, VSS, FS, FSS> {
	pub fn vertex_input<T>(self, vertex_input: T) -> GraphicalPassBuilder<T, VS, VSS, FS, FSS> {
		GraphicalPassBuilder {
			vertex_input: vertex_input,
			vertex_shader: self.vertex_shader,
			primitive_topology: self.primitive_topology,
			rasterization: self.rasterization,
			fragment_shader: self.fragment_shader,
		}
	}

	pub fn single_buffer_input<V>(self) -> GraphicalPassBuilder<SingleBufferDefinition<V>, VS, VSS, FS, FSS> {
		self.vertex_input(SingleBufferDefinition::<V>::new())
	}

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
		}
	}

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
		}
	}
}

impl<V, VS, FS> GraphicalPassBuilder<SingleBufferDefinition<V>, VS, VS::SpecializationConstants, FS, FS::SpecializationConstants>
where
	VS : GraphicsEntryPointAbstract,
	FS : GraphicsEntryPointAbstract,
	VS::PipelineLayout : Send + Sync + Clone + 'static,
	FS::PipelineLayout : Send + Sync + Clone + 'static,
	SingleBufferDefinition<V> : VertexDefinition<VS::InputDefinition>,
{
	pub fn build_present_pass(self, device: &Device) -> Result<GraphicalPass<impl GraphicsPipelineAbstract, impl RenderPassAbstract, (), PresentPass>, BuildError> {
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
					format: device.swapchain_depth_format,
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {depth}
			})?);

		let pipeline = Arc::new(GraphicsPipeline::start()
			.vertex_input_single_buffer::<V>()
			.vertex_shader(self.vertex_shader.0, self.vertex_shader.1)
			.triangle_list()
			.cull_mode_back()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(self.fragment_shader.0, self.fragment_shader.1)
			.depth_stencil_simple_depth()
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.device.clone())?);
		
		Ok(GraphicalPass { render_pass, pipeline, images: (), phantom: std::marker::PhantomData })
	}
}

impl From<RenderPassCreationError> for BuildError {
	fn from(err: RenderPassCreationError) -> Self { Self::RenderPassCreation(err) }
}
impl From<GraphicsPipelineCreationError> for BuildError {
	fn from(err: GraphicsPipelineCreationError) -> Self { Self::GraphicsPipelineCreation(err) }
}