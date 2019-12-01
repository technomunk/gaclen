//! A lit, textured Cube example.
//! 
//! This example showcases all functionality required to render a believable 3D world.
//! It is limited to a single cube, a real scene would be a lot more complex, likely with additional helper code and resources.
//! 
//! Please note, that because of screen-space coordinate mismatch between OpenGL and Vulkan the `up` coordinate and triangle-faces are reversed.

extern crate gaclen;

mod shaders;
mod geometry;

use gaclen::graphics;

use gaclen::window::{
	WindowBuilder,
	EventsLoop,
	Event, WindowEvent,
};

use std::sync::Arc;

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let mut events_loop = EventsLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Cube example")
			.with_dimensions((1280, 720).into())
			.with_min_dimensions((1280, 720).into())
			.build(&events_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let mut device = graphics::device::Device::new(&context, window.clone(), graphics::device::PresentMode::Immediate).unwrap();
	println!("Initialized device: {:?}", device);

	let albedo_pass = {
		let vs = shaders::vertex::Shader::load(&device).unwrap();
		let fs = shaders::fragment::Shader::load(&device).unwrap();

		graphics::pass::GraphicalPass::start()
			.single_buffer_input::<geometry::Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.fragment_shader(fs.main_entry_point(), ())
			.basic_depth_test()
			.front_face_clockwise()
			.cull_back()
			.build_present_pass(&device).unwrap()
	};

	let geometry = geometry::generate_cube(&device).unwrap();

	let transform_buffer_pool = device.create_cpu_buffer_pool::<shaders::vertex::ty::TransformData>(graphics::BufferUsage::all());
	let light_buffer_pool = device.create_cpu_buffer_pool::<shaders::fragment::ty::LightData>(graphics::BufferUsage::all());
	
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

		let transform = {
			let data = transform_from_time(start_time.elapsed().as_secs_f32(), window.get_inner_size().unwrap().into());
			transform_buffer_pool.next(data).unwrap()
		};
		let light = {
			let data = shaders::fragment::ty::LightData {
				position: [2.0, -1.0, 2.0],
				_dummy0: [0; 4],
				direct: [1.0; 3],
				_dummy1: [0; 4],
				ambient: [0.1, 0.0, 0.0],
			};
			light_buffer_pool.next(data).unwrap()
		};

		let descriptor_set = Arc::new(albedo_pass.start_persistent_descriptor_set(0)
			.add_buffer(transform).unwrap()
			.add_buffer(light).unwrap()
			.build().unwrap());

		let after_frame = device.begin_frame().unwrap()
			.begin_pass(&albedo_pass, vec![clear_color.into(), 1f32.into()])
				.draw(vec![geometry.clone()], descriptor_set, ())
				.finish_pass()
			.finish_frame();
		
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

fn transform_from_time(time: f32, window_resolution: (u32, u32)) -> shaders::vertex::ty::TransformData {
	let rotation = time / 10.0 * std::f32::consts::PI;

	let aspect = window_resolution.0 as f32 / window_resolution.1 as f32;

	let model: cgmath::Matrix4<f32> = cgmath::Euler{ x: cgmath::Rad(0.0), y: cgmath::Rad(0.0), z: cgmath::Rad(rotation) }.into();
	let proj: cgmath::Matrix4<f32> = cgmath::PerspectiveFov { fovy: cgmath::Deg(40.0).into(), aspect, near: 0.1, far: 5.0 }.into();

	shaders::vertex::ty::TransformData {
		model: model.into(),
		view: cgmath::Matrix4::look_at(
			cgmath::Point3 { x: 3.0, y: 0.0, z: 2.0 },
			cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
			cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 }).into(),
		proj: proj.into(),
	}
}