extern crate gaclen;

use gaclen::graphics;

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
			.with_title("Triangle example")
			.with_dimensions((1280, 720).into())
			.with_min_dimensions((1280, 720).into())
			.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone()).unwrap();
	println!("Initialized device: {:?}", device);

	let mut pass = graphics::pipeline::Pass::new(&device, &window).unwrap();
	let mut recreate_swapchain = false;

	let triangle_buffer = graphics::buffer::triangle(&device) as std::sync::Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>;
	
	let mut previous_frame_end = Some(Box::new(device.get_frame_end()) as Box<dyn vulkano::sync::GpuFuture>);

	let mut running = true;
	while running {
		previous_frame_end.as_mut().unwrap().cleanup_finished();

		if recreate_swapchain {
			device.window_resized(&window).unwrap();
			pass.resize_for_window(&device, &window).unwrap();
			recreate_swapchain = false;
		}

		let clear_color = [0.0, 0.0, 0.0, 1.0];
		let push_constants = push_constants_from_time(start_time.elapsed().as_secs_f64());
		let (commands, acquire_future, image_num) = device.build_draw_command_buffer(&pass, vec![triangle_buffer.clone()], clear_color, push_constants).unwrap();

		let prev = previous_frame_end.take();
		let after_commands_built = prev.unwrap().join(acquire_future);

		let after_draw = device.execute_after(after_commands_built, commands).unwrap();
		let after_present = device.present_after(after_draw, image_num);
		let after_flush = device.flush_after(after_present);

		match after_flush {
			Ok(future) => previous_frame_end = Some(Box::new(future)),
			Err(vulkano::sync::FlushError::OutOfDate) => {
				recreate_swapchain = true;
				previous_frame_end = Some(Box::new(device.get_frame_end()));
			},
			Err(err) => {
				println!("Error presenting: {:?}", err);
				previous_frame_end = Some(Box::new(device.get_frame_end()));
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

fn push_constants_from_time(time: f64) -> graphics::shader::vertex::ty::PushConstantData {
	let color = (time.sin() * 0.5 + 0.5) as f32;
	let rotation = time as f32 % (2.0 * std::f32::consts::PI);
	
	graphics::shader::vertex::ty::PushConstantData {
		color_rotation: [1.0, color, 0.0, rotation].into(),
	}
}