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

	let mut pass = graphics::pass::AlbedoPass::new(&device, &window).unwrap();
	let mut recreate_swapchain = false;

	let triangle_buffer = graphics::buffer::triangle(&device) as std::sync::Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>;
	
	let mut previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>> = None;

	let mut running = true;
	while running {
		if let Some(time) = &mut previous_frame_end {
			time.cleanup_finished();
		}

		if recreate_swapchain {
			device.resize_for_window(&window).unwrap();
			pass.resize_for_window(&device, &window).unwrap();
			recreate_swapchain = false;
		}

		let clear_color = [0.0, 0.0, 0.0, 1.0];
		let push_constants = push_constants_from_time(start_time.elapsed().as_secs_f64());
		
		let (updated_device, after_frame) = device.start_frame(previous_frame_end, &pass, vec![clear_color.into()]).unwrap()
			.draw(&pass, vec![triangle_buffer.clone()], push_constants)
			.finish_frame();
		
		device = updated_device;

		match after_frame {
			Ok(future) => previous_frame_end = Some(future),
			Err(graphics::device::FrameFinishError::Flush(vulkano::sync::FlushError::OutOfDate)) => {
				recreate_swapchain = true;
				previous_frame_end = None;
			},
			Err(err) => {
				println!("Error drawing: {:?}", err);
				previous_frame_end = None;
			}
		};

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