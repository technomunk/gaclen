// This file is NOT exported by the crate
// It is used to manually test the functionality of the engine

mod graphics;
mod window;

use vulkano::sync::GpuFuture;

use winit::{
	WindowBuilder,
	EventsLoop,
	Event, WindowEvent,
};

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let mut events_loop = EventsLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
		.with_title("gaclen")
		.with_dimensions((1280, 720).into())
		// .with_resizable(false)
		.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone()).unwrap();
	println!("Initialized device: {:?}", device);

	let mut pass = graphics::pipeline::Pass::new(&device, &window).unwrap();
	let mut recreate_swapchain = false;

	let triangle_buffer = graphics::buffer::triangle(&device);

	let mut previous_frame_end = Some(device.get_frame_end());

	let mut running = true;
	while running {
		previous_frame_end.as_mut().unwrap().cleanup_finished();

		if recreate_swapchain {
			device.window_resized(&window).unwrap();
			pass.resize_for_window(&device, &window).unwrap();
			recreate_swapchain = false;
		}

		let buffers = vec!(triangle_buffer.clone());
		let (commands, acquire_future, image_num) = device.build_draw_command_buffer(&pass, &buffers).unwrap();

		let prev = previous_frame_end.take();
		let after_commands_built = prev.unwrap().join(acquire_future);

		let after_draw = device.execute_after(after_commands_built, commands).unwrap();
		let after_present = device.present_after(after_draw, image_num);
		let after_flush = device.flush_after(after_present);

		match after_flush {
			Ok(future) => previous_frame_end = Some(Box::new(future)),
			Err(vulkano::sync::FlushError::OutOfDate) => {
				recreate_swapchain = true;
				previous_frame_end = Some(device.get_frame_end());
			},
			Err(err) => {
				println!("Error presenting: {:?}", err);
				previous_frame_end = Some(device.get_frame_end());
			}
		}

		frame_count = frame_count + 1;

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