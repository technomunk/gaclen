//! A phong-lit, textured Cube example.
//! 
//! This example showcases all functionality required to render a believable 3D world.
//! It is limited to a single cube, a real scene would be a lot more complex, likely with additional helper code and resources.
//! 
//! Please note, that because of screen-space coordinate mismatch between OpenGL and Vulkan the `up` coordinate and triangle-faces are reversed.

// Allow `shader!` macro to use this project's gaclen dependency.
extern crate gaclen;

mod shaders;
mod geometry;

use gaclen::graphics;
use gaclen::winit;

use cgmath::One;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

use std::sync::Arc;

fn main() {
	let mut frame_count: u64 = 0;
	let start_time = std::time::Instant::now();

	let event_loop = EventLoop::new();
	let window = std::sync::Arc::new(
		WindowBuilder::new()
			.with_title("Cube example")
			.with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.with_min_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
			.build(&event_loop).unwrap()
	);
	
	let context = graphics::context::Context::new().unwrap();
	let device = graphics::device::Device::new(&context).unwrap();
	println!("Initialized device: {:?}", device);
	let mut swapchain = graphics::swapchain::Swapchain::new(&context, &device, window.clone(), graphics::swapchain::PresentMode::Immediate, graphics::image::Format::D16Unorm).expect("Failed to create swapchain!");

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
			.add_image_attachment_swapchain_cleared(&swapchain)
			.add_depth_attachment_swapchain_discard(&swapchain, graphics::pass::LoadOp::Clear).unwrap()
			.build(&device).unwrap()
	};

	let geometry = geometry::generate_cube(&device).unwrap();

	let transform_buffer_pool = graphics::buffer::CpuBufferPool::<shaders::vertex::ty::TransformData>::new(device.logical_device(), graphics::buffer::BufferUsage::all());
	let light_buffer_pool = graphics::buffer::CpuBufferPool::<shaders::fragment::ty::LightData>::new(device.logical_device(), graphics::buffer::BufferUsage::all());

	let texture = {
		let image = image::open("gaclen/examples/phong_cube/texture.png").unwrap().to_rgba();
		let (width, height) = image.dimensions();
        let dimensions = graphics::image::Dimensions::Dim2d { width, height };
		let image_data = image.into_raw(); // to_rgba() returns Vec<u8> backed container
		
		graphics::image::create_immutable_image_from_iter(&device, image_data.iter().cloned(), dimensions, graphics::image::Format::R8G8B8A8Srgb).unwrap()
	};

	let sampler = graphics::image::Sampler::simple_repeat_linear(device.logical_device());
	
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
		.build().unwrap()
	);

	let mut recreate_swapchain = false;

	let mut rotation_enabled = false;
	let mut last_x = 0;
	let mut last_y = 0;
	let mut object_rotation = cgmath::Quaternion::one();

	// Wrap the device in a stack-allocated container to allow for temporary ownership.
	let mut device = Some(device);

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
				*control_flow = ControlFlow::Exit;
				let run_duration = start_time.elapsed().as_secs_f64();
				let fps: f64 = frame_count as f64 / run_duration;
				println!("Produced {} frames over {:.2} seconds ({:.2} avg fps)", frame_count, run_duration, fps);
			},
			Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
			Event::WindowEvent { event: WindowEvent::MouseInput{state, button, .. }, .. } => {
				rotation_enabled = (button == winit::event::MouseButton::Right) && state == winit::event::ElementState::Pressed;
			}
			Event::WindowEvent { event: WindowEvent::CursorMoved{ position, .. }, .. } => {
				let (x, y) = position.into();
				
				if rotation_enabled {
					let (width, height) : (f64, f64) = window.inner_size().into();
					let delta_x = (x as f32 - last_x as f32) / width as f32;
					let delta_y = (y as f32 - last_y as f32) / height as f32;
					let delta : cgmath::Quaternion<_> = cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(delta_y), -cgmath::Rad(delta_x)).into();
					object_rotation = delta * object_rotation;
				}

				last_x = x;
				last_y = y;
			},
			Event::RedrawEventsCleared => {
				if recreate_swapchain {
					let dimensions = window.inner_size();
					match swapchain.resize(dimensions.into()) {
						Ok(()) => (),
						Err(graphics::ResizeError::Swapchain(_)) => {
							println!("Failed to resize window, skipping frame!");
							return;
						},
						Err(err) => panic!(err),
					};
					recreate_swapchain = false;
				}
		
				let clear_color = [0.0, 0.0, 0.0, 1.0];
		
				let transform = {
					let data = transform(object_rotation.clone(), window.inner_size().into());
					transform_buffer_pool.next(data).unwrap()
				};
		
				let transform_descriptor_set = Arc::new(albedo_pass.start_persistent_descriptor_set(0)
					.add_buffer(transform).unwrap()
					.build().unwrap()
				);
		
				// Device ownership is taken here.
				let frame = graphics::frame::Frame::begin(device.take().unwrap(), &swapchain).unwrap();
		
				let framebuffer = std::sync::Arc::new(albedo_pass.start_framebuffer()
					.add(swapchain.get_color_image_for(&frame)).unwrap()
					.add(swapchain.get_depth_image_for(&frame)).unwrap()
					.build().unwrap()
				);
		
				let after_frame = frame.begin_pass(&albedo_pass, framebuffer, swapchain.default_viewport(), vec![clear_color.into(), 1f32.into()])
					.draw(vec![geometry.clone()], (transform_descriptor_set, light_descriptor_set.clone()), ())
					.finish_pass()
				.finish();
				
				// Return device.
				device = match after_frame {
					Ok(device) => Some(device),
					Err((device, err)) => {
						if err == graphics::frame::FrameFinishError::Flush(gaclen::graphics::vulkano::sync::FlushError::OutOfDate) { recreate_swapchain = true; };
						Some(device)
					},
				};
		
				frame_count += 1;
			},
			_ => ()
		}
	});
}

// Ideally the view and projection matrices would be found by some 'Camera' concept.
fn transform(rotation: cgmath::Quaternion<f32>, viewport_dimensions: (u32, u32)) -> shaders::vertex::ty::TransformData {
	let aspect = viewport_dimensions.0 as f32 / viewport_dimensions.1 as f32;

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
