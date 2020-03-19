//! [Swapchains](https://vulkan.lunarg.com/doc/view/1.0.26.0/linux/tutorial/html/05-init_swapchain.html) are sets of images that are used to draw and present pictures to the screen and the configuration for how to present them.
//! 
//! Main usage for *swapchains* is in [`Frame`](struct.Frame.html) [initialization](struct.Frame.html#method.begin) and they determine the resolution of the image that will be drawn.
//! To draw an image that will then be presented use [`Swapchain::get_image_for()`](struct.Swapchain.html#method.get_color_image_for) when building the [`Framebuffer`](struct.Framebuffer.html) for a pass that will draw presented results.
//! 
//! **Gaclen**'s [`Swapchain`s](struct.Swapchain.html) currently also include [depth buffers](https://en.wikipedia.org/wiki/Z-buffering) that match the size of the image, this functionality however might change in the near future.

use super::ResizeError;
use super::context::Context;
use super::device::Device;
use super::frame::Frame;

use winit::window::Window;

use std::sync::Arc;

use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device as LogicalDevice, Queue as DeviceQueue};
use vulkano::format::Format;
use vulkano::image::{AttachmentImage, SwapchainImage, ImageCreationError};
use vulkano::swapchain::{Surface, Swapchain as VlkSwapchain, SwapchainCreationError as VlkSwapchainCreationError};
use vulkano::pipeline::viewport::Viewport;

pub use vulkano::swapchain::PresentMode;

type ImageFormat = (Format, vulkano::swapchain::ColorSpace);

/// Swapchain is the infrastructure for drawing on the screen.
/// 
/// It includes the front and back buffers that are presented on the screen.
pub struct Swapchain {
	pub(super) device: Arc<LogicalDevice>,

	pub(super) swapchain: Arc<VlkSwapchain<Arc<Window>>>,
	pub(super) images: Vec<Arc<SwapchainImage<Arc<Window>>>>,
	pub(super) depths: Vec<Arc<AttachmentImage>>,
	pub(super) depth_format: Format,
	pub(super) inverse_depth: bool,

	pub(super) dynamic_state: DynamicState,
}

/// An error during the creation of a [`Swapchain`](struct.Swapchain.html).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwapchainCreationError {
	/// Error during the creation of draw-surface.
	Surface(vulkano::swapchain::SurfaceCreationError),
	/// Error during querying draw-surface capabilities.
	SurfaceCapabilities(vulkano::swapchain::CapabilitiesError),
	/// Error during the creation of the swapchain.
	Swapchain(VlkSwapchainCreationError),
	/// Error during the creation of the depth-buffer image.
	Image(ImageCreationError),
	/// No applicable format for draw-surface was found.
	NoCompatibleFormatFound,
	/// Window passed for the creation of the device has no apparent size..
	UnsizedWindow,
}

impl Swapchain {
	/// Create a new Swapchain using provided Device.
	pub fn new(
		context: &Context,
		device: &Device,
		window: Arc<Window>,
		present_mode: PresentMode,
		depth_format: Format,
	) -> Result<Swapchain, SwapchainCreationError>
	{
		let logical_device = device.logical_device();

		let dimensions: (u32, u32) = window.inner_size().into();
		let surface = vulkano_win::create_vk_surface(window, context.instance.clone())?;
		let (swapchain, images) = create_swapchain(device, surface, dimensions, &device.graphics_queue, present_mode)?;

		let depths = {
			let image_count = images.len();
			let mut images = Vec::with_capacity(image_count);
			for _ in 0..image_count {
				images.push(AttachmentImage::transient(logical_device.clone(), [dimensions.0, dimensions.1], depth_format)?);
			};
			images
		};

		let mut dynamic_state = DynamicState::default();
		resize_dynamic_state_viewport(&mut dynamic_state, dimensions, false);

		Ok(Swapchain {
			device: logical_device,
			swapchain,
			images,
			depths,
			depth_format,
			inverse_depth: false,
			dynamic_state,
		})
	}

	/// Set the depth buffer to use forward (inverse == false) or inverse range.
	/// 
	/// Forward range is 0.0 being the front and the 1.0 being the away.
	/// Inverse range is 1.0 the front and 0.0 being the away.
	/// The advantages of different approaches are to be researched by the reader.
	pub fn inverse_depth(&mut self, inverse: bool) {
		self.inverse_depth = inverse;
		let dimensions = {
			let dimensions = self.depths[0].dimensions();
			(dimensions[0], dimensions[1])
		};
		resize_dynamic_state_viewport(&mut self.dynamic_state, dimensions, inverse);
	}

	/// Resize the images in the swapchain to provided size.
	pub fn resize(&mut self, dimensions: (u32, u32)) -> Result<(), ResizeError> {
		resize_dynamic_state_viewport(&mut self.dynamic_state, dimensions, self.inverse_depth);

		// TODO: investigate weird UnsupportedDimensions swapchain error on some resizes
		let (swapchain, images) = self.swapchain.recreate_with_dimensions([dimensions.0, dimensions.1])?;
		self.swapchain = swapchain;
		self.images = images;

		self.depths = {
			let image_count = self.images.len();
			let mut images = Vec::with_capacity(image_count);
			for _ in 0..image_count {
				images.push(AttachmentImage::transient(self.device.clone(), [dimensions.0, dimensions.1], self.depth_format)?);
			};
			images
		};

		Ok(())
	}

	/// Get the target image to draw to for provided frame.
	pub fn get_color_image_for(&self, frame: &Frame) -> Arc<SwapchainImage<Arc<Window>>> {
		self.images[frame.swapchain_index].clone()
	}

	/// Get the target depth image to draw to for provided frame.
	pub fn get_depth_image_for(&self, frame: &Frame) -> Arc<AttachmentImage> {
		self.depths[frame.swapchain_index].clone()
	}
}

impl From<vulkano::swapchain::SurfaceCreationError> for SwapchainCreationError {
	fn from(err: vulkano::swapchain::SurfaceCreationError) -> Self { Self::Surface(err) }
}
impl From<ImageCreationError> for SwapchainCreationError {
	fn from(err: ImageCreationError) -> Self { Self::Image(err) }
}

fn create_swapchain(
	device: &Device,
	surface: Arc<Surface<Arc<Window>>>,
	dimensions: (u32, u32),
	graphics_queue: &Arc<DeviceQueue>,
	present_mode: PresentMode
) -> Result<(Arc<VlkSwapchain<Arc<Window>>>, Vec<Arc<SwapchainImage<Arc<Window>>>>), SwapchainCreationError> {
	let capabilities = match surface.capabilities(device.physical_device()) {
		Ok(caps) => caps,
		Err(err) => return Err(SwapchainCreationError::SurfaceCapabilities(err)),
	};
	let usage = capabilities.supported_usage_flags;
	let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();

	let (format, color_space) = select_format(capabilities.supported_formats)?;

	let swapchain = VlkSwapchain::new(
		device.logical_device(),
		surface,
		capabilities.min_image_count,
		format,
		[dimensions.0, dimensions.1],
		1,
		usage,
		graphics_queue,
		vulkano::swapchain::SurfaceTransform::Identity,
		alpha,
		present_mode,
		vulkano::swapchain::FullscreenExclusive::Default,
		true,
		color_space
	);
	
	match swapchain {
		Ok(swapchain) => Ok(swapchain),
		Err(err) => Err(SwapchainCreationError::Swapchain(err)),
	}
}

fn select_format(formats: Vec<ImageFormat>) -> Result<ImageFormat, SwapchainCreationError> {
	if formats.is_empty() {
		return Err(SwapchainCreationError::NoCompatibleFormatFound);
	}

	let mut format = formats[0];

	for other in formats {
		format = choose_better_format(format, other);
	}
	Ok(format)
}

fn choose_better_format(first: ImageFormat, _second: ImageFormat) -> ImageFormat {
	// TODO: compare and select better format
	first
}

fn resize_dynamic_state_viewport(dynamic_state: &mut DynamicState, dimensions: (u32, u32), inverse: bool) {
	let viewport = Viewport {
		origin: [0.0, 0.0],
		dimensions: [dimensions.0 as f32, dimensions.1 as f32],
		depth_range: if inverse { 1.0 .. 0.0 } else { 0.0 .. 1.0 },
	};
	
	match dynamic_state.viewports {
		Some(ref mut vec) => {
			match vec.len() {
				0 => vec.push(viewport),
				_ => vec[0] = viewport,
			}
		},
		None => dynamic_state.viewports = Some(vec![viewport]),
	};
}
