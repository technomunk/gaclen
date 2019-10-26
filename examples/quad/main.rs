extern crate gaclen;

mod shaders;

use gaclen::graphics;

use winit::{
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
			.with_title("Triangle example")
			.with_dimensions((1280, 720).into())
			.with_min_dimensions((1280, 720).into())
			.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone()).unwrap();
	println!("Initialized device: {:?}", device);

	let mut pass = {
		let vs = shaders::vertex::Shader::load(device.logical_device()).unwrap();
		let fs = shaders::fragment::Shader::load(device.logical_device()).unwrap();

		graphics::pass::AlbedoPass::new::<_, _, Vertex>(&device, &window, vs.main_entry_point(), (), fs.main_entry_point(), ()).unwrap()
	};

	let triangle_buffer = device.create_buffer([
		Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.25, 0.75, 0.25, 1.0 ] },
		Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 0.75, 0.25, 0.25, 1.0 ] },
		Vertex { position: [ 0.5, 0.5, 0.0 ], color: [ 0.75, 0.75, 0.25, 0.0 ] },

		Vertex { position: [-0.5,-0.5, 0.0 ], color: [ 0.0, 0.0, 0.0, 1.0 ] },
		Vertex { position: [ 0.5,-0.5, 0.0 ], color: [ 1.0, 0.0, 0.0, 1.0 ] },
		Vertex { position: [-0.5, 0.5, 0.0 ], color: [ 0.0, 1.0, 0.0, 1.0 ] },
	].iter().cloned()).unwrap();

	let mut recreate_swapchain = false;
	
	let mut previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>> = None;

	let mut running = true;
	while running {
		if recreate_swapchain {
			device.resize_for_window(&window).unwrap();
			pass.resize_for_window(&device, &window).unwrap();
			recreate_swapchain = false;
		}

		let clear_color = [0.0, 0.0, 0.0, 1.0];
		let push_constants = push_constants_from_time(start_time.elapsed().as_secs_f32(), window.get_inner_size().unwrap().into());
		
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

fn push_constants_from_time(time: f32, window_resolution: (u32, u32)) -> shaders::vertex::ty::PushConstantData {
	let time = time / 5.0;

	let x = time.cos() * 0.0;
	let y = time.sin() * 0.0 - 1.0;

	let view = cgmath::Matrix4::look_at(
		cgmath::Point3 { x, y, z: 1.0 },
		cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
		cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0 });
	
	let aspect = window_resolution.0 as f32 / window_resolution.1 as f32;
	
	let proj: cgmath::Matrix4<f32> = cgmath::PerspectiveFov { fovy: cgmath::Deg(60.0).into(), aspect, near: std::f32::EPSILON, far: 4.0 }.into();

	let mvp = proj * view;

	shaders::vertex::ty::PushConstantData { MVP: mvp.into() }
}