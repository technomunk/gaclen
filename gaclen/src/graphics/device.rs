//! The device uses GPU to execute rendering and computing commands.
//! 
//! In order to get an image using a [Device](struct.Device.html) one needs to:
//! 1. [Create it](struct.Device.html#method.new).
//! 2. [Create a render-pass](trait.Pass.html).
//! 3. [Create a data-buffer](struct.Device.html#method.create_buffer).
//! 4. [Start a frame](struct.Device.html#method.start_frame).
//! 5. [Draw using a pass](struct.DrawingDevice.html#method.draw).
//! 6. [Finish the frame](struct.DrawingDevice.html#method.finish_frame).
//! 
//! Note that during the middle of the frame the device switches to [DrawingDevice](struct.DrawingDevice.html) struct, which represents 'middle of frame' state.

use crate::window::Window;
use super::context::Context;
use super::ResizeError;
use super::pass::{GraphicalPass, PresentPass};

use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer};
use vulkano::format::Format;
use vulkano::image::{AttachmentImage, ImageCreationError};
use vulkano::framebuffer::{Framebuffer, RenderPassAbstract};
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferExecError, DynamicState};
use vulkano::device::{Device as LogicalDevice, DeviceExtensions, Queue as DeviceQueue};
use vulkano::image::SwapchainImage;
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreationError};
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::pipeline::GraphicsPipelineAbstract;

pub use vulkano::swapchain::PresentMode;

type ImageFormat = (vulkano::format::Format, vulkano::swapchain::ColorSpace);

/// A device responsible for hardware-accelerated computations.
/// 
/// It is responsible for recording, submitting and synchronizing commands and data to the GPU.
pub struct Device {
	pub(super) device: Arc<LogicalDevice>,

	pub(super) graphics_queue: Arc<DeviceQueue>,
	pub(super) transfer_queue: Arc<DeviceQueue>,
	pub(super) compute_queue: Arc<DeviceQueue>,

	pub(super) swapchain: Arc<Swapchain<Arc<Window>>>,
	pub(super) swapchain_images: Vec<Arc<SwapchainImage<Arc<Window>>>>,
	pub(super) swapchain_depths: Vec<Arc<AttachmentImage>>,
	pub(super) swapchain_depth_format: Format,
	pub(super) inverse_depth: bool,

	pub(super) dynamic_state: DynamicState,

	pub(super) last_frame: Option<Box<dyn GpuFuture>>,
}

/// A device that is in the middle of drawing a frame.
pub struct Frame {
	device: Device,
	time: Box<dyn GpuFuture>,
	commands: AutoCommandBufferBuilder,
	image_index: usize,
}

/// A device that is in the middle of a draw-pass in a middle of drawing a frame.
pub struct PassInFrame<'a, P : ?Sized, RP : ?Sized, PP> {
	frame: Frame,
	pass: &'a GraphicalPass<P, RP, PP>,
}

/// Error during device creation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceCreationError {
	/// No hardware devices were found.
	NoPhysicalDevicesFound,
	/// Some hardware devices was found, but none of it was applicable for gaclen.
	NoCompatiblePhysicalDeviceFound,
	/// Error during the creation of logical device.
	Logical(vulkano::device::DeviceCreationError),
	/// Error during the creation of draw-surface.
	Surface(vulkano::swapchain::SurfaceCreationError),
	/// Error during querying draw-surface capabilities.
	SurfaceCapabilities(vulkano::swapchain::CapabilitiesError),
	/// Error during the creation of the swapchain.
	Swapchain(SwapchainCreationError),
	/// Error during the creation of the depth-buffer image.
	Image(ImageCreationError),
	/// No applicable format for draw-surface was found.
	NoCompatibleFormatFound,
	/// Window passed for the creation of the device has no apparent size..
	UnsizedWindow,
}

/// Error finishing the frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameFinishError {
	/// Error during flushing commands to the GPU.
	Flush(FlushError),
	/// Error during attempted execution of GPU commands.
	Commands(CommandBufferExecError),
}

impl Device {
	// TODO: add create() function with all parameters and use with_* for setting individual non-default parameters and new() with all default parameters.

	/// Create a new device with default parameters.
	pub fn new(context: &Context, window: Arc<Window>, present_mode: PresentMode) -> Result<Device, DeviceCreationError> {
		let physical = select_physical_device(context)?;

		let device_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
		let queues = select_queue_families(&physical);
		let (logical, queues) = LogicalDevice::new(physical, physical.supported_features(), &device_extensions, queues.iter().cloned())?;
		let [graphics_queue, transfer_queue, compute_queue] = unpack_queues(queues.collect());

		let dimensions: (u32, u32) = match window.get_inner_size() {
			Some(size) => size.into(),
			None => return Err(DeviceCreationError::UnsizedWindow),
		};
		let surface = vulkano_win::create_vk_surface(window, context.instance.clone())?;
		let (swapchain, swapchain_images) = create_swapchain(physical, logical.clone(), surface, dimensions, &graphics_queue, present_mode)?;

		let swapchain_depth_format = Format::D16Unorm;

		let swapchain_depths = {
			let image_count = swapchain_images.len();
			let mut images = Vec::with_capacity(image_count);
			for _ in 0..image_count {
				images.push(AttachmentImage::transient(logical.clone(), [dimensions.0, dimensions.1], swapchain_depth_format)?);
			};
			images
		};

		let dynamic_state = DynamicState::default();

		let mut device = Device {
			device: logical,
			graphics_queue,
			transfer_queue,
			compute_queue,
			swapchain,
			swapchain_images,
			swapchain_depths,
			swapchain_depth_format,
			inverse_depth: false,
			dynamic_state,
			last_frame: None,
		};

		resize_dynamic_state_viewport(&mut device.dynamic_state, dimensions, false);

		Ok(device)
	}

	/// Set the depth buffer to use forward (inverse == false) or inverse range.
	/// 
	/// Forward range is 0.0 being the front and the 1.0 being the away.
	/// Inverse range is 1.0 the front and 0.0 being the away.
	/// The advantages of different approaches are to be researched by the reader.
	pub fn inverse_depth(&mut self, inverse: bool) {
		self.inverse_depth = inverse;
		let dimensions = {
			let dimensions = self.swapchain_depths[0].dimensions();
			(dimensions[0], dimensions[1])
		};
		resize_dynamic_state_viewport(&mut self.dynamic_state, dimensions, inverse);
	}

	/// Update the device for the resized window.
	pub fn resize_for_window(&mut self, window: &Window) -> Result<(), ResizeError> {
		let dimensions: (u32, u32) = match window.get_inner_size() {
			Some(size) => size.into(),
			None => return Err(ResizeError::UnsizedWindow),
		};

		resize_dynamic_state_viewport(&mut self.dynamic_state, dimensions, self.inverse_depth);

		// TODO: investigate weird UnsupportedDimensions swapchain error on some resizes
		let (swapchain, images) = self.swapchain.recreate_with_dimension([dimensions.0, dimensions.1])?;
		self.swapchain = swapchain;
		self.swapchain_images = images;

		self.swapchain_depths = {
			let image_count = self.swapchain_images.len();
			let mut images = Vec::with_capacity(image_count);
			for _ in 0..image_count {
				images.push(AttachmentImage::transient(self.device.clone(), [dimensions.0, dimensions.1], self.swapchain_depth_format)?);
			};
			images
		};

		Ok(())
	}

	/// Begin drawing the frame.
	/// 
	/// Takes ownership of the [Device](struct.Device.html) and converts it to a [Frame](struct.Frame.html).
	/// This corresponds to switching to the special 'middle of frame' state, that caches intermediate commands before submitting them to the GPU.
	/// In order to draw after beginning the frame, one needs to [begin a pass](struct.Frame.html#method.begin_pass).
	#[inline]
	pub fn begin_frame(mut self) -> Result<Frame, (Self, vulkano::swapchain::AcquireError)> {
		let (image_index, image_acquire_time) = match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
			Ok(result) => result,
			Err(err) => return Err((self, err)),
		};

		let time: Box<dyn GpuFuture> = match self.last_frame.take() {
			Some(mut time) => {
				time.cleanup_finished();
				Box::new(time.join(image_acquire_time))
			},
			None => Box::new(vulkano::sync::now(self.device.clone()).join(image_acquire_time)),
		};

		let commands = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.graphics_queue.family()).unwrap();

		let frame = Frame {
			device: self,
			time,
			commands,
			image_index,
		};
		Ok(frame)
	}

	/// Create a basic buffer for data for processing on the GPU.
	pub fn create_buffer<T: 'static>(&self, data_iterator: impl ExactSizeIterator<Item = T>) -> Result<Arc<CpuAccessibleBuffer<[T]>>, vulkano::memory::DeviceMemoryAllocError> {
		CpuAccessibleBuffer::from_iter(self.device.clone(), vulkano::buffer::BufferUsage::all(), data_iterator)
	}

	/// Get the underlying logical device (useful for supplying to own shaders).
	pub fn logical_device(&self) -> Arc<LogicalDevice> { self.device.clone() }
}

#[cfg(feature = "expose-underlying-vulkano")]
impl Device {
	/// Get the [vulkano device queue](DeviceQueue) used for graphical operations.
	#[inline(always)]
	pub fn graphics_queue(&self) -> &Arc<DeviceQueue> { self.graphics_queue }
	/// Get the [vulkano device queue](DeviceQueue) used for transfer operations.
	#[inline(always)]
	pub fn transfer_queue(&self) -> &Arc<DeviceQueue> { self.transfer_queue }
	/// Get the [vulkano device queue](DeviceQueue) used for compute operations.
	#[inline(always)]
	pub fn compute_queue(&self) -> &Arc<DeviceQueue> { self.compute_queue }
	/// Get the [vulkano swapchian](Swapchain) used for presenting images on the screen.
	#[inline(always)]
	pub fn swapchain(&self) -> &Arc<Swapchain<Arc<Window>>> { self.swapchain }
	/// Get the [vulkano swapchain images](SwapchainImage) that are presented on the screen.
	#[inline(always)]
	pub fn swapchain_images(&self) -> &Vec<Arc<SwapchainImage<Arc<Window>>>> { self.swapchain_images }
}

impl Frame {
	/// Begin a PresentPass (the results will be visible on the screen).
	pub fn begin_pass<'a, P, RP>(
		mut self,
		pass: &'a GraphicalPass<P, RP, PresentPass>,
		clear_values: Vec<vulkano::format::ClearValue>)
	-> PassInFrame<'a, P, RP, PresentPass>
	where
		P : ?Sized,
		RP : ?Sized + RenderPassAbstract + Send + Sync + 'static,
	{
		let framebuffer = Framebuffer::start(pass.render_pass.clone())
			.add(self.device.swapchain_images[self.image_index].clone()).unwrap()
			.add(self.device.swapchain_depths[self.image_index].clone()).unwrap()
			.build().unwrap();
		self.commands = self.commands.begin_render_pass(Arc::new(framebuffer), false, clear_values).unwrap();

		PassInFrame {
			frame: self,
			pass: pass,
		}
	}

	/// Finish drawing the frame and flush the commands to the GPU.
	/// 
	/// Releases the Device to allow starting a new frame, allocate new resources and anything else a [Device] is able to do.
	#[inline]
	pub fn finish_frame(self) -> Result<Device, (Device, FrameFinishError)> {
		let commands = self.commands.build().unwrap();
		let after_execute = match self.time.then_execute(self.device.graphics_queue.clone(), commands) {
			Ok(future) => future,
			Err(err) => return Err((self.device, FrameFinishError::Commands(err))),
		};

		let after_flush = after_execute.then_swapchain_present(self.device.graphics_queue.clone(), self.device.swapchain.clone(), self.image_index)
			.then_signal_fence_and_flush();
		
		let after_frame = match after_flush {
			Ok(future) => future,
			Err(err) => return Err((self.device, FrameFinishError::Flush(err))),
		};
		let device = Device { last_frame: Some(Box::new(after_frame)), .. self.device };
		Ok(device)
	}
}

impl<'a, P, RP> PassInFrame<'a, P, RP, PresentPass>
where
	P : ?Sized + GraphicsPipelineAbstract + Send + Sync + 'static,
	RP : ?Sized,
{
	/// Draw some data using a pass.
	/// 
	/// The result depends highly on the [GraphicalPass](traits.GraphicalPass.html) that was used to create the [PassInFrame].
	/// Push-constants should correspond to the ones in the shader used for creating the [GraphicalPass](traits.GraphicalPass.html).
	#[inline]
	pub fn draw<PC>(
		mut self,
		vertex_buffer: Vec<Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>>,
		push_constants: PC
	) -> Self {
		self.frame.commands = self.frame.commands.draw(self.pass.pipeline.clone(), &self.frame.device.dynamic_state, vertex_buffer, (), push_constants).unwrap();
		self
	}

	/// Finish using a GraphicalPass.
	/// 
	/// Releases the consumed [Frame] to begin the next pass or finish the frame.
	#[inline]
	pub fn finish_pass(self) -> Frame {
		let commands = self.frame.commands.end_render_pass().unwrap();
		Frame { commands, .. self.frame }
	}
}

impl From<vulkano::device::DeviceCreationError> for DeviceCreationError {
	fn from(err: vulkano::device::DeviceCreationError) -> DeviceCreationError { DeviceCreationError::Logical(err) }
}
impl From<vulkano::swapchain::SurfaceCreationError> for DeviceCreationError {
	fn from(err: vulkano::swapchain::SurfaceCreationError) -> DeviceCreationError { DeviceCreationError::Surface(err) }
}
impl From<ImageCreationError> for DeviceCreationError {
	fn from(err: ImageCreationError) -> DeviceCreationError { DeviceCreationError::Image(err) }
}

impl std::fmt::Debug for Device {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(fmt, "{{")?;
		let physical_device = self.device.physical_device();
		writeln!(fmt, "  name: {}", physical_device.name())?;
		writeln!(fmt, "  type: {:?}", physical_device.ty())?;
		writeln!(fmt, "  graphics_queue:")?;
		write_queue_details(fmt, &self.graphics_queue, "    ")?;
		writeln!(fmt)?;
		writeln!(fmt, "  transfer_queue:")?;
		write_queue_details(fmt, &self.transfer_queue, "    ")?;
		writeln!(fmt)?;
		writeln!(fmt, "  compute_queue:")?;
		write_queue_details(fmt, &self.compute_queue, "    ")?;
		write!(fmt, "}}")?;
		Ok(())
	}
}


fn select_physical_device(context: &Context) -> Result<PhysicalDevice, DeviceCreationError> {
	let mut devices = PhysicalDevice::enumerate(&context.instance);
	let mut device = match devices.next() {
		Some(device) => device,
		None => return Err(DeviceCreationError::NoPhysicalDevicesFound),
	};

	for other in devices { device = choose_better_device(device, other); };
	
	match validate_physical_device(&device) {
		true => Ok(device),
		false => Err(DeviceCreationError::NoCompatiblePhysicalDeviceFound),
	}
}

fn create_swapchain(
	physical_device: PhysicalDevice,
	logical_device: Arc<LogicalDevice>,
	surface: Arc<Surface<Arc<Window>>>,
	dimensions: (u32, u32),
	graphics_queue: &Arc<DeviceQueue>,
	present_mode: vulkano::swapchain::PresentMode
) -> Result<(Arc<Swapchain<Arc<Window>>>, Vec<Arc<SwapchainImage<Arc<Window>>>>), DeviceCreationError> {
	let capabilities = match surface.capabilities(physical_device) {
		Ok(caps) => caps,
		Err(err) => return Err(DeviceCreationError::SurfaceCapabilities(err)),
	};
	let usage = capabilities.supported_usage_flags;
	let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();

	let format = select_format(capabilities.supported_formats)?;

	let swapchain = Swapchain::new(
		logical_device,
		surface,
		capabilities.min_image_count,
		format.0,
		[dimensions.0, dimensions.1],
		1,
		usage,
		graphics_queue,
		vulkano::swapchain::SurfaceTransform::Identity,
		alpha,
		present_mode,
		true,
		None
	);
	
	match swapchain {
		Ok(swapchain) => Ok(swapchain),
		Err(err) => Err(DeviceCreationError::Swapchain(err)),
	}
}


fn select_format(formats: Vec<ImageFormat>) -> Result<ImageFormat, DeviceCreationError> {
	if formats.is_empty() { return Err(DeviceCreationError::NoCompatibleFormatFound); }

	let mut format = formats[0];

	for other in formats {
		format = choose_better_format(format, other);
	}
	Ok(format)
}

fn select_queue_families<'a>(device: &PhysicalDevice<'a>) -> Vec<(vulkano::instance::QueueFamily<'a>, f32)> {
	let mut families = device.queue_families();
	let first = families.next().unwrap();

	let mut graphics = first.clone();
	let mut transfer = first.clone();
	let mut compute = first;

	for other in families {
		graphics = choose_better_graphics_family(graphics, other.clone());
		transfer = choose_better_transfer_family(transfer, other.clone());
		compute = choose_better_compute_family(compute, other);
	};

	// Hacky cast abuse, append if the queues_count is larger than number of collisions
	let append_transfer = transfer.queues_count() > (transfer.id() == graphics.id()) as usize;
	let append_compute = compute.queues_count() > (compute.id() == graphics.id() || compute.id() == transfer.id()) as usize + append_transfer as usize;

	let mut result = Vec::new();
	result.push((graphics, 1.0));
	if append_transfer { result.push((transfer, 0.5)); }
	if append_compute { result.push((compute, 0.25)); }

	result
}

fn unpack_queues(mut queues: Vec<Arc<DeviceQueue>>) -> [Arc<DeviceQueue>; 3] {
	match queues.len() {
		1 => {
			let q = queues.pop().unwrap();
			[q.clone(), q.clone(), q]
		},
		// TODO: implement unpacking 2 queues
		2 => panic!("Unimplemented unpack_queues for just 2 queues, bug Griffone!"),
		3 => {
			// TODO: make sure the queues are able to do the thing they were supposed to!
			let compute = queues.pop().unwrap();
			let transfer = queues.pop().unwrap();
			let graphics = queues.pop().unwrap();
			[graphics, transfer, compute]
		},
		_ => panic!("Unexpected number of queues created, something wend wrong during device initialization.")
	}
}

fn resize_dynamic_state_viewport(dynamic_state: &mut DynamicState, dimensions: (u32, u32), inverse: bool) {
	let viewport = Viewport {
		origin: [ 0.0, dimensions.1 as f32],
		dimensions: [dimensions.0 as f32, -(dimensions.1 as f32)],
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

fn validate_physical_device<'a>(device: &PhysicalDevice<'a>) -> bool {
	if device.api_version() < super::REQUIRED_VULKAN_VERSION { return false; }

	let mut supports_graphics = false;
	let mut supports_compute = false;

	for family in device.queue_families() {
		supports_graphics = supports_graphics || (family.queues_count() > 0 && family.supports_graphics());
		supports_compute = supports_compute || (family.queues_count() > 0 && family.supports_compute());
	};

	supports_compute && supports_graphics
}

fn choose_better_device<'a>(first: PhysicalDevice<'a>, second: PhysicalDevice<'a>) -> PhysicalDevice<'a> {
	if !validate_physical_device(&second) { return first; };

	// TODO: compare and select best device
	first
}

fn choose_better_format(first: ImageFormat, _second: ImageFormat) -> ImageFormat {
	// TODO: compare and select better format
	first
}

fn choose_better_graphics_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.supports_graphics() { return first; };

	// prefer exclusively graphics queue
	match second.supports_compute() {
		true => first,
		false => second
	}
}

fn choose_better_transfer_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.explicitly_supports_transfers() { return first; };

	match second.supports_graphics() {
		true => first,
		false => match first.supports_graphics() {
			true => second,
			false => match second.supports_compute() {
				true => first,
				false => second,
			},
		},
	}
}

fn choose_better_compute_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.supports_compute() { return first; };

	match second.supports_graphics() {
		true => first,
		false => second
	}
}

fn write_queue_details(fmt: &mut std::fmt::Formatter, queue: &DeviceQueue, prefix: &str) -> std::fmt::Result {
	let family = queue.family();
	writeln!(fmt, "{}id: {}-{}", prefix, family.id(), queue.id_within_family())?;
	writeln!(fmt, "{}graphics: {}", prefix, family.supports_graphics())?;
	writeln!(fmt, "{}transfer: {}", prefix, family.explicitly_supports_transfers())?;
	writeln!(fmt, "{}compute: {}", prefix, family.supports_compute())?;
	Ok(())
}