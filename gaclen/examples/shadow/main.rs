extern crate gaclen;

mod shaders;
mod geometry;

use gaclen::graphics;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

use std::sync::Arc;

#[derive(Default, Debug, Clone)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, color);

const SHADOW_TEXTURE_SIDE: u32 = 512;

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let event_loop = EventLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Shadowing example")
			.with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.with_min_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.build(&event_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let device = graphics::device::Device::new(&context).unwrap();
	println!("Initialized device: {:?}", device);
	let mut swapchain = graphics::swapchain::Swapchain::new(&context, &device, window.clone(), graphics::swapchain::PresentMode::Immediate, graphics::image::Format::D16Unorm).expect("Failed to create swapchain!");

	let shadow_pass = {
		let vs = shaders::shadow::vertex::Shader::load(&device).unwrap();
		let fs = shaders::shadow::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.cull_front()
			.basic_depth_test()
			.add_depth_attachment(graphics::image::Format::D32Sfloat, graphics::pass::LoadOp::Clear, graphics::pass::StoreOp::Store).unwrap()
			.build(&device).unwrap()
	};

	let albedo_pass = {
		let vs = shaders::albedo::vertex::Shader::load(&device).unwrap();
		let fs = shaders::albedo::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.cull_back()
			.basic_depth_test()
			.add_image_attachment_swapchain_cleared(&swapchain)
			.add_depth_attachment_swapchain_discard(&swapchain, graphics::pass::LoadOp::Clear).unwrap()
			.build(&device).unwrap()
	};

	let quad = geometry::generate_quad(&device);
	let cube = geometry::generate_cube(&device);

	let mut recreate_swapchain = false;

	let plane_matrix_buffer = graphics::buffer::CpuAccessibleBuffer::from_data(
		device.logical_device(),
		graphics::buffer::BufferUsage::all(),
		false,
		[
			[ [ 2.0f32, 0.0, 0.0, 0.0 ], [ 0.0, 2.0, 0.0, 0.0 ], [ 0.0, 0.0, 2.0, 0.0 ], [ 0.0, 0.0, 0.0, 1.0 ] ],
		]
	).unwrap();

	let cube_matrix_buffer = graphics::buffer::CpuAccessibleBuffer::from_data(
		device.logical_device(),
		graphics::buffer::BufferUsage::all(),
		false,
		[
			[ [ 1.0f32, 0.0, 0.0, 0.0 ], [ 0.0, 1.0, 0.0, 0.0 ], [ 0.0, 0.0, 1.0, 0.0 ], [ 0.0, 0.0, 1.5, 1.0 ] ],
		]
	).unwrap();

	let light_matrix_buffer = graphics::buffer::CpuAccessibleBuffer::from_data(
		device.logical_device(),
		graphics::buffer::BufferUsage::all(),
		false,
		generate_shadow_matrix()
	).unwrap();

	let plane_buffer_descriptor = Arc::new(
		albedo_pass.start_persistent_descriptor_set(0)
			.add_buffer(plane_matrix_buffer).unwrap()
			.build().unwrap());
	
	let cube_buffer_descriptor = Arc::new(
		albedo_pass.start_persistent_descriptor_set(0)
			.add_buffer(cube_matrix_buffer).unwrap()
			.build().unwrap());

	let light_buffer_descriptor = Arc::new(
		albedo_pass.start_persistent_descriptor_set(1)
			.add_buffer(light_matrix_buffer).unwrap()
			.build().unwrap());

	let shadow_image = Arc::new(graphics::image::AttachmentImage::sampled(device.logical_device(), [SHADOW_TEXTURE_SIDE; 2], graphics::image::Format::D32Sfloat).unwrap());

	let shadow_sampler = graphics::image::Sampler::compare(
		device.logical_device(),
		graphics::image::Filter::Linear,
		graphics::image::Filter::Linear,
		graphics::image::MipmapMode::Nearest,
		graphics::image::SamplerAddressMode::ClampToBorder(graphics::image::BorderColor::FloatOpaqueWhite),
		graphics::image::SamplerAddressMode::ClampToBorder(graphics::image::BorderColor::FloatOpaqueWhite),
		graphics::image::SamplerAddressMode::ClampToBorder(graphics::image::BorderColor::FloatOpaqueWhite),
		0f32,
		1f32,
		0f32,
		0f32,
		graphics::image::Compare::Greater).unwrap();

	let shadow_descriptor = Arc::new(
		albedo_pass.start_persistent_descriptor_set(2)
			.add_sampled_image(shadow_image.clone(), shadow_sampler).unwrap()
			.build().unwrap());

	// Wrap the device in a stack-allocated container to allow for temporary ownership.
	let mut device = Some(device);

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
				*control_flow = ControlFlow::Exit;
				let run_duration = start_time.elapsed().as_secs_f64();
				let fps: f64 = frame_count as f64 / run_duration;
				println!("Produced {} frames over {:.2} seconds ({:.2} avg fps)", frame_count, run_duration, fps);
				return;
			},
			Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
			Event::RedrawEventsCleared => {
				if recreate_swapchain {
					let dimensions = window.inner_size();

					// Sometimes the swapchain fails to create :(
					match swapchain.resize(dimensions.into()) {
						Ok(()) => (),
						Err(graphics::ResizeError::Swapchain(_)) => {
							println!("Failed to resize window, skipping frame!");
							return;
						},
						Err(err) => panic!(err),
					};
					recreate_swapchain = false;
				}

				let clear_color = [0.1, 0.1, 0.3, 1.0];

				let frame = graphics::frame::Frame::begin(device.take().unwrap(), &swapchain).unwrap();

				let shadow_framebuffer = Arc::new(shadow_pass.start_framebuffer()
					.add(shadow_image.clone()).unwrap()
					.build().unwrap());

				let albedo_framebuffer = Arc::new(albedo_pass.start_framebuffer()
					.add(swapchain.get_color_image_for(&frame)).unwrap()
					.add(swapchain.get_depth_image_for(&frame)).unwrap()
					.build().unwrap());

				let camera_matrix = generate_camera_matrix(window.inner_size().into());
				// let camera_matrix = shaders::albedo::vertex::ty::PushConstantData { view_projection_matrix: generate_shadow_matrix() };

				let after_frame = {
					frame
						.begin_pass_with_viewport(&shadow_pass, shadow_framebuffer, vec![1f32.into()], graphics::frame::Viewport{ origin: [0f32; 2], dimensions: [SHADOW_TEXTURE_SIDE as f32; 2], depth_range: 0f32..1f32 })
							.draw(vec![cube.clone()], (cube_buffer_descriptor.clone(), light_buffer_descriptor.clone()), ())
						.finish_pass()
						.begin_pass(&albedo_pass, albedo_framebuffer, vec![clear_color.into(), 1f32.into()])
							.draw(vec![quad.clone()], (plane_buffer_descriptor.clone(), light_buffer_descriptor.clone(), shadow_descriptor.clone()), camera_matrix)
							.draw(vec![cube.clone()], (cube_buffer_descriptor.clone(), light_buffer_descriptor.clone(), shadow_descriptor.clone()), camera_matrix)
						.finish_pass()
					.finish()
				};
				
				device = match after_frame {
					Ok(device) => Some(device),
					Err((device, err)) => {
						if err == graphics::frame::FrameFinishError::Flush(vulkano::sync::FlushError::OutOfDate) { recreate_swapchain = true; };
						Some(device)
					},
				};

				frame_count += 1;
			}
			_ => ()
		};
	});
}


fn generate_camera_matrix(viewport_dimensions: (u32, u32)) -> shaders::albedo::vertex::ty::PushConstantData {
	let aspect = viewport_dimensions.0 as f32 / viewport_dimensions.1 as f32;

	let proj: cgmath::Matrix4<f32> = cgmath::PerspectiveFov { fovy: cgmath::Deg(50.0).into(), aspect, near: 1.0, far: 9.0 }.into();
	let view: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at(
		cgmath::Point3 { x: 0.0, y: -3.0, z: 3.0 },
		cgmath::Point3 { x: 0.0, y: 0.0, z: 1.0 },
		cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 });

	shaders::albedo::vertex::ty::PushConstantData {
		view_projection_matrix: (proj * view).into()
	}
}

fn generate_shadow_matrix() -> [[f32; 4]; 4] {
	let proj: cgmath::Matrix4<f32> = cgmath::ortho(-1.0, 1.0, -1.0, 1.0, 1.0, 9.0);

	let view: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at(
		cgmath::Point3 { x: 3.0, y: 3.0, z: 6.0 },
		cgmath::Point3 { x: 0.0, y: 0.0, z: 1.5 },
		cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 });
	
	(proj * view).into()
}
