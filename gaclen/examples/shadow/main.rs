extern crate gaclen;

mod shaders;
mod geometry;

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
vulkano::impl_vertex!(Vertex, position, color);

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let mut events_loop = EventsLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Shadowing example")
			.with_dimensions((1280, 720).into())
			.with_min_dimensions((1280, 720).into())
			.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone(), graphics::device::PresentMode::Immediate).unwrap();
	println!("Initialized device: {:?}", device);

	let albedo_pass = {
		let vs = shaders::albedo::vertex::Shader::load(&device).unwrap();
		let fs = shaders::albedo::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.build_present_pass(&device).unwrap()
	};

	let quad = geometry::generate_quad(&device);
	let cube = geometry::generate_quad(&device);

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

		let after_frame = device.begin_frame().unwrap()
			.begin_pass(&albedo_pass, vec![clear_color.into(), 1f32.into()])
				.draw(vec![quad.clone()], ())
				.draw(vec![cube.clone()], ())
				.finish_pass()
			.finish_frame();
		
		device = match after_frame {
			Ok(device) => device,
			Err((device, err)) => {
				if err == graphics::device::FrameFinishError::Flush(vulkano::sync::FlushError::OutOfDate) { recreate_swapchain = true; };
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