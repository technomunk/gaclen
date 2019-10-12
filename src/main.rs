// This file is NOT exported by the crate
// It is used to manually test the functionality of the engine

mod graphics;

use winit::{
	WindowBuilder,
	EventsLoop,
	Event, WindowEvent,
};

fn main() {
	let mut events_loop = EventsLoop::new();
	let window = WindowBuilder::new()
		.with_title("gaclen")
		.with_dimensions((1280, 720).into())
		.with_resizable(false)
		.build(&events_loop).unwrap();
	
	let context = graphics::context::Context::new().unwrap();
	let device = graphics::device::Device::new(&context).unwrap();

	// let surface = vulkano_win::create_vk_surface(window, vulkan_instance).unwrap();

	let mut running = true;
	while running {
		events_loop.poll_events(|event| {
			match event {
				Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { running = false; },
				_ => ()
			}
		});
	}
}