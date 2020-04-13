extern crate gaclen;

mod scene;
mod graphics;

use gaclen::winit::event::{Event, WindowEvent};
use gaclen::winit::event_loop::{ControlFlow, EventLoop};
use gaclen::winit::window::WindowBuilder;

use std::sync::Arc;

fn main() {
	let event_loop = EventLoop::new();

	let window = Arc::new(WindowBuilder::new()
		.with_title("Scene Example")
		.with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
		.with_min_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
		.build(&event_loop).unwrap()
	);

	let mut graphics_system = graphics::GraphicsSystem::new(window.clone());

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
			_ => (),
		}
	});
}
