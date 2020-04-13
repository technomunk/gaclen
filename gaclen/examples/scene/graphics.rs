//! Application-specific system for drawing a scene.

mod shaders;

use gaclen::graphics::buffer::{BufferAccess};
use gaclen::graphics::context::Context as GaclenContext;
use gaclen::graphics::device::Device as GaclenDevice;
use gaclen::graphics::image::{Format as ImageFormat};
use gaclen::graphics::swapchain::{Swapchain as GaclenSwapchain, PresentMode};

use std::sync::Arc;

pub struct GraphicsSystem {
	context: GaclenContext,
	device: GaclenDevice,
	swapchain: GaclenSwapchain,
	models: Vec<Arc<dyn BufferAccess>>,
}

impl GraphicsSystem {
	/// Initialize the graphics system for a given window.
	pub fn new(window: Arc<gaclen::winit::window::Window>) -> Self {
		let context = GaclenContext::new().expect("Failed to create graphical context. Try updating graphics drivers!");
		let device = GaclenDevice::new(&context).expect("Failed to find a capable device!");
		let swapchain = GaclenSwapchain::new(&context, &device, window, PresentMode::Immediate, ImageFormat::D24Unorm_S8Uint).expect("Failed to initialize a swapchain!");

		Self {
			context,
			device,
			swapchain,
		}
	}
}
