use crate::window::Window;
use super::device::Device;
use super::ResizeError;

use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, RenderPassCreationError, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineCreationError};
use vulkano::command_buffer::DynamicState;

use std::sync::Arc;


// Pass is a stage in the rendering pipeline that takes some inputs, does some processing with provided shaders and provides some output
pub struct Pass {
	pub(super) render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pub(super) graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	pub(super) framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

	// TODO: not all passes required dynamics state, only dynamic ones
	pub(super) dynamic_state: DynamicState,
}

// Pipeline is a collection of Passes and their dependencies that allows execution of commands in a defined ordering on GPU
pub struct Pipeline {
	// TODO: populate
}

#[derive(Debug)]
pub enum PassCreationError {
	RenderPass(RenderPassCreationError), // Error during creation of the underlying vulkan render-pass
	GraphicsPipeline(GraphicsPipelineCreationError), // Error during creation of the underlying vulkan graphics-pipeline
	DynamicState(ResizeError), // Error during initial resizing
}


impl Pass {
	// Create a new Pass that uses provided shaders
	pub fn new(
		device: &Device,
		window: &Arc<Window>
	) -> Result<Pass, PassCreationError> {
		let render_pass = Arc::new(vulkano::single_pass_renderpass!(
			device.device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: device.swapchain.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		)?);

		let vs = super::shader::vertex::Shader::load(device.device.clone()).unwrap();
		let fs = super::shader::fragment::Shader::load(device.device.clone()).unwrap();

		let graphics_pipeline = Arc::new(GraphicsPipeline::start()
			.vertex_input_single_buffer::<super::buffer::Vertex2D>()
			.vertex_shader(vs.main_entry_point(), ())
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fs.main_entry_point(), ())
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.device.clone())?);
		
		let mut pass = Pass {
			render_pass,
			graphics_pipeline,
			framebuffers: Vec::new(),
			dynamic_state: DynamicState::default(),
		};
		pass.resize_for_window(device, window)?;
		Ok(pass)
	}

	pub fn resize_for_window(&mut self, device: &Device, window: &Arc<Window>) -> Result<(), ResizeError> {
		let dimensions: (u32, u32) = match window.get_inner_size() {
			Some(size) => size.into(),
			None => return Err(ResizeError::UnsizedWindow)
		};
		
		let viewport = vulkano::pipeline::viewport::Viewport {
			origin: [0.0, 0.0],
			dimensions: [dimensions.0 as f32, dimensions.1 as f32],
			depth_range: 0.0 .. 1.0,
		};

		self.dynamic_state.viewports = Some(vec!(viewport));

		self.framebuffers = device.swapchain_images.iter().map(|image| {
			Arc::new(
				vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
					.add(image.clone()).unwrap()
					.build().unwrap()
			) as Arc<dyn FramebufferAbstract + Send + Sync>
		}).collect::<Vec<_>>();
		Ok(())
	}
}


impl From<RenderPassCreationError> for PassCreationError {
	fn from(err: RenderPassCreationError) -> PassCreationError { PassCreationError::RenderPass(err) }
}
impl From<GraphicsPipelineCreationError> for PassCreationError {
	fn from(err: GraphicsPipelineCreationError) -> PassCreationError { PassCreationError::GraphicsPipeline(err) }
}
impl From<ResizeError> for PassCreationError {
	fn from(err: ResizeError) -> PassCreationError { PassCreationError::DynamicState(err) }
}