use super::device::Device;

use std::sync::Arc;

use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

// TODO: add own pipeline object

fn basic_render_pass<W>(device: &Device<W>) -> Arc<dyn RenderPassAbstract> {
	Arc::new(vulkano::single_pass_renderpass!(
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
	).unwrap())
}

pub fn basic_draw_pipeline<W>(device: &Device<W>) -> Arc<dyn GraphicsPipelineAbstract> {
	let vs = super::shader::vertex::Shader::load(device.device.clone()).unwrap();
	let fs = super::shader::fragment::Shader::load(device.device.clone()).unwrap();

	Arc::new(GraphicsPipeline::start()
		.vertex_input_single_buffer::<super::buffer::Vertex2D>()
		.vertex_shader(vs.main_entry_point(), ())
		.triangle_list()
		.viewports_dynamic_scissors_irrelevant(1)
		.fragment_shader(fs.main_entry_point(), ())
		.render_pass(Subpass::from(basic_render_pass(&device), 0).unwrap())
		.build(device.device.clone())
		.unwrap())
}