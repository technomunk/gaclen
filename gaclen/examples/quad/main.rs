//! Most basic gaclen usage example.
//! 
//! Draws a single quad without depth or culling.
//! 
//! Please note, that because of screen-space coordinate mismatch between OpenGL and Vulkan the `up` coordinate is reversed.

extern crate gaclen;

mod shaders;

use gaclen::graphics;

use gaclen::window::{
	WindowBuilder,
	EventsLoop,
	Event, WindowEvent,
};

#[derive(Default, Debug, Clone)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 4],
}
gaclen::graphics::impl_vertex!(Vertex, position, color);

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let mut events_loop = EventsLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Quad example")
			.with_dimensions((1280, 720).into())
			.with_min_dimensions((1280, 720).into())
			.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone(), graphics::device::PresentMode::Immediate).unwrap();
	println!("Initialized device: {:?}", device);

	let pass = {
		let vs = shaders::vertex::Shader::load(&device).unwrap();
		let fs = shaders::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.add_attachment_swapchain_image(&device, graphics::pass::LoadOp::Clear)
			.add_attachment_swapchain_depth_discard(&device, graphics::pass::LoadOp::Clear).unwrap()
			.build(&device).unwrap()
	};

	let triangle_buffer = device.create_cpu_accessible_buffer([
		Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.25, 0.75, 0.25, 1.0 ] },
		Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 0.75, 0.25, 0.25, 1.0 ] },
		Vertex { position: [ 0.5, 0.5, 0.0 ], color: [ 0.75, 0.75, 0.25, 0.0 ] },

		Vertex { position: [-0.5,-0.5, 0.0 ], color: [ 0.0, 0.0, 0.0, 1.0 ] },
		Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 1.0, 0.0, 0.0, 1.0 ] },
		Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.0, 1.0, 0.0, 1.0 ] },
	].iter().cloned()).unwrap();

	let mut recreate_swapchain = false;

	let mut running = true;
	while running {
		if recreate_swapchain {
			// Sometimes the swapchain fails to create :(
			match device.resize_for_window(&window) {
				Ok(()) => (),
				Err(graphics::ResizeError::Swapchain(_)) => {
					println!("Failed to resize window, skipping frame!");
					continue;
				},
				Err(err) => panic!(err),
			};
			recreate_swapchain = false;
		}

		let clear_color = [0.0, 0.0, 0.0, 1.0];
		let push_constants = push_constants_from_time(start_time.elapsed().as_secs_f32(), window.get_inner_size().unwrap().into());

		let frame = device.begin_frame().unwrap();

		let framebuffer = std::sync::Arc::new(pass.start_framebuffer()
			.add(frame.get_swapchain_image()).unwrap()
			.add(frame.get_swapchain_depth()).unwrap()
			.build().unwrap()
		);

		let after_frame = frame.begin_pass(&pass, framebuffer, vec![clear_color.into(), 1.0f32.into()])
			.draw(vec![triangle_buffer.clone()], (), push_constants)
			.finish_pass().finish_frame();
		
		device = match after_frame {
			Ok(device) => device,
			Err((device, err)) => {
				if err == graphics::device::FrameFinishError::Flush(gaclen::graphics::vulkano::sync::FlushError::OutOfDate) { recreate_swapchain = true; };
				device
			},
		};

		frame_count += 1;

		events_loop.poll_events(|event| {
			match event {
				Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => running = false,
				Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
				_ => ()
			}
		});
	}

	let run_duration = start_time.elapsed().as_secs_f64();
	let fps: f64 = frame_count as f64 / run_duration;

	println!("Produced {} frames over {:.2} seconds ({:.2} avg fps)", frame_count, run_duration, fps);
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
