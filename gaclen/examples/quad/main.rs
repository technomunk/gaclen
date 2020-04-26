//! Most basic gaclen usage example.
//! 
//! Draws a single quad without depth or culling.
//! 
//! Please note, that because of screen-space coordinate mismatch between OpenGL and Vulkan the `up` coordinate is reversed.

// Allow `shader!` macro to use this project's gaclen dependency.
extern crate gaclen;

mod shaders;

use gaclen::graphics;
use gaclen::winit;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

#[derive(Default, Debug, Clone)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 4],
}
gaclen::graphics::impl_vertex!(Vertex, position, color);

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let event_loop = EventLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Quad example")
			.with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.with_min_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.build(&event_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let device = graphics::device::Device::new(&context).unwrap();
	println!("Initialized device: {:?}", device);
	let mut swapchain = graphics::swapchain::Swapchain::new(&context, &device, window.clone(), graphics::swapchain::PresentMode::Immediate, graphics::image::Format::D16Unorm).expect("Failed to create swapchain!");

	let pass = {
		let vs = shaders::vertex::Shader::load(&device).unwrap();
		let fs = shaders::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.add_image_attachment_swapchain_cleared(&swapchain)
			.add_depth_attachment_swapchain_discard(&swapchain, graphics::pass::LoadOp::Clear).unwrap()
			.build(&device).unwrap()
	};

	let triangle_buffer = graphics::buffer::CpuAccessibleBuffer::from_data(
		device.logical_device(),
		graphics::buffer::BufferUsage::all(),
		false,
		[
			Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.25, 0.75, 0.25, 1.0 ] },
			Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 0.75, 0.25, 0.25, 1.0 ] },
			Vertex { position: [ 0.5, 0.5, 0.0 ], color: [ 0.75, 0.75, 0.25, 0.0 ] },

			Vertex { position: [-0.5,-0.5, 0.0 ], color: [ 0.0, 0.0, 0.0, 1.0 ] },
			Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 1.0, 0.0, 0.0, 1.0 ] },
			Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.0, 1.0, 0.0, 1.0 ] },
		]
	).unwrap();

	let mut recreate_swapchain = false;

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
			},
			Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
			Event::RedrawEventsCleared => {
				if recreate_swapchain {
					let dimensions = window.inner_size();
		
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
		
				let clear_color = [0.0, 0.0, 0.0, 1.0];
				let push_constants = push_constants_from_time(start_time.elapsed().as_secs_f32(), window.inner_size().into());
		
				let frame = graphics::frame::Frame::begin(device.take().unwrap(), &swapchain).unwrap();
		
				let framebuffer = std::sync::Arc::new(pass.start_framebuffer()
					.add(swapchain.get_color_image_for(&frame)).unwrap()
					.add(swapchain.get_depth_image_for(&frame)).unwrap()
					.build().unwrap()
				);
		
				let after_frame = frame.begin_pass(&pass, framebuffer, swapchain.default_viewport(), vec![clear_color.into(), 1.0f32.into()])
					.draw(vec![triangle_buffer.clone()], (), push_constants)
					.finish_pass()
				.finish();
				
				device = match after_frame {
					Ok(device) => Some(device),
					Err((device, err)) => {
						if err == graphics::frame::FrameFinishError::Flush(gaclen::graphics::vulkano::sync::FlushError::OutOfDate) { recreate_swapchain = true; };
						Some(device)
					},
				};
		
				frame_count += 1;
			}
			_ => ()
		}
	});
}

fn push_constants_from_time(time: f32, window_resolution: (u32, u32)) -> shaders::vertex::ty::PushConstantData {
	let time = time / 5.0;

	let x = time.cos();
	let y = time.sin();

	let view = cgmath::Matrix4::look_at(
		cgmath::Point3 { x, y, z: 1.0 },
		cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
		cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 });
	
	let aspect = window_resolution.0 as f32 / window_resolution.1 as f32;
	
	let proj: cgmath::Matrix4<f32> = cgmath::PerspectiveFov { fovy: cgmath::Deg(60.0).into(), aspect, near: 0.1, far: 4.0 }.into();

	let mvp = proj * view;

	shaders::vertex::ty::PushConstantData { MVP: mvp.into() }
}
