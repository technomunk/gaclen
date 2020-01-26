//! A phong-lit, textured Cube example.
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

	let texture = {
		let image = image::open("gaclen/examples/phong_cube/texture.png").unwrap().to_rgba();
		let (width, height) = image.dimensions();
        let dimensions = graphics::Dimensions::Dim2d { width, height };
		let image_data = image.into_raw(); // to_rgba() returns Vec<u8> backed container
		
		device.create_immutable_image_from_iter(image_data.iter().cloned(), dimensions, graphics::PixelFormat::R8G8B8A8Srgb).unwrap()
	};

	let sampler = device.create_simple_linear_repeat_sampler().unwrap();
	
	let light = {
		let data = shaders::fragment::ty::LightData {
			position: [2.0, -2.0, 1.0, 0.0],
			ambient: [0.1; 4],
			diffuse: [0.8, 0.8, 0.8, 2.0],
			specular: [1.0; 4], // 4th component is power
		};
		light_buffer_pool.next(data).unwrap()
	};

	let light_descriptor_set = Arc::new(albedo_pass.start_persistent_descriptor_set(1)
		.add_buffer(light).unwrap()
		.add_sampled_image(texture, sampler).unwrap()
		.build().unwrap());

	let mut recreate_swapchain = false;

	let mut rotation_enabled = false;
	let mut last_x = 0;
	let mut last_y = 0;
	let mut object_rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);

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
			let data = transform(object_rotation.clone(), window.get_inner_size().unwrap().into());
			transform_buffer_pool.next(data).unwrap()
		};

		let transform_descriptor_set = Arc::new(albedo_pass.start_persistent_descriptor_set(0)
			.add_buffer(transform).unwrap()
			.build().unwrap());

		let after_frame = device.begin_frame().unwrap()
			.begin_pass(&albedo_pass, vec![vulkano::format::ClearValue::None, 1f32.into()])
				.draw(vec![geometry.clone()], (transform_descriptor_set, light_descriptor_set.clone()), ())
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
				Event::WindowEvent { event: WindowEvent::MouseInput{state, button, .. }, .. } => {
					rotation_enabled = (button == gaclen::window::MouseButton::Right) && state == gaclen::window::ElementState::Pressed;

				}
				Event::WindowEvent { event: WindowEvent::CursorMoved{ position, .. }, .. } => {
					let (x, y) = position.into();
					
					if rotation_enabled {
						let (width, height) : (f64, f64) = window.get_inner_size().unwrap().into();
						let delta_x = (x as f32 - last_x as f32) / width as f32;
						let delta_y = (y as f32 - last_y as f32) / height as f32;
						let delta : cgmath::Quaternion<_> = cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(delta_y), -cgmath::Rad(delta_x)).into();
						object_rotation = delta * object_rotation;
					}

					last_x = x;
					last_y = y;
				}
				_ => ()
			}
		});
	}

	let run_duration = start_time.elapsed().as_secs_f64();
	let fps: f64 = frame_count as f64 / run_duration;

	println!("Produced {} frames over {:.2} seconds ({:.2} avg fps)", frame_count, run_duration, fps);
}

// Ideally the view and projection matrices would be found by some 'Camera' concept.
fn transform(rotation: cgmath::Quaternion<f32>, window_resolution: (u32, u32)) -> shaders::vertex::ty::TransformData {
	let aspect = window_resolution.0 as f32 / window_resolution.1 as f32;

	let model: cgmath::Matrix4<f32> = rotation.into();
	let proj: cgmath::Matrix4<f32> = cgmath::PerspectiveFov { fovy: cgmath::Deg(40.0).into(), aspect, near: 0.1, far: 4.0 }.into();

	shaders::vertex::ty::TransformData {
		model: model.into(),
		view: cgmath::Matrix4::look_at(
			cgmath::Point3 { x: 3.0, y: 0.0, z: 0.0 },
			cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
			cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 }).into(),
		proj: proj.into(),
	}
}