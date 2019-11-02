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
use super::pass::GraphicalPass;

use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferExecError};
use vulkano::device::{Device as LogicalDevice, DeviceExtensions, Queue as DeviceQueue};
use vulkano::image::SwapchainImage;
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreationError};
use vulkano::sync::{GpuFuture, FlushError};

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
}

/// A device that is in the middle of drawing a frame.
pub struct DrawingDevice {
	device: Device,
	time: Box<dyn GpuFuture>,
	commands: AutoCommandBufferBuilder,
	image_index: usize,
}

/// Error during device creation.
#[derive(Debug)]
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
	/// No applicable format for draw-surface was found.
	NoCompatibleFormatFound,
	/// Window passed for the creation of the device has no apparent size..
	UnsizedWindow,
}

/// Error finishing the frame.
#[derive(Debug)]
pub enum FrameFinishError {
	/// Error during flushing commands to the GPU.
	Flush(FlushError),
	/// Error during attempted execution of GPU commands.
	Commands(CommandBufferExecError),
}

impl Device {
	/// Create a new device that targets a specific window.
	pub fn new(context: &Context, window: Arc<Window>) -> Result<Device, DeviceCreationError> {
		let physical = select_physical_device(context)?;

		let device_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
		let queues = select_queue_families(&physical);
		let (logical, queues) = LogicalDevice::new(physical, physical.supported_features(), &device_extensions, queues.iter().cloned())?;
		let [graphics_queue, transfer_queue, compute_queue] = unpack_queues(queues.collect());

		let dimensions = match window.get_inner_size() {
			Some(size) => size,
			None => return Err(DeviceCreationError::UnsizedWindow),
		};
		let surface = vulkano_win::create_vk_surface(window, context.instance.clone())?;
		let (swapchain, swapchain_images) = create_swapchain(physical, logical.clone(), surface, dimensions.into(), &graphics_queue)?;

		let device = Device {
			device: logical,
			graphics_queue,
			transfer_queue,
			compute_queue,
			swapchain,
			swapchain_images,
		};

		Ok(device)
	}

	/// Update the device for the resized window.
	pub fn resize_for_window(&mut self, window: &Window) -> Result<(), ResizeError> {
		let dimensions: (u32, u32) = match window.get_inner_size() {
			Some(size) => size.into(),
			None => return Err(ResizeError::UnsizedWindow),
		};

		let (swapchain, images) = self.swapchain.recreate_with_dimension([dimensions.0, dimensions.1])?;
		self.swapchain = swapchain;
		self.swapchain_images = images;
		Ok(())
	}

	/// Start drawing the frame.
	/// 
	/// Takes ownership of the [Device](struct.Device.html) and converts it to a [DrawingDevice](struct.DrawingDevice.html).
	/// This corresponds to switching to the special 'middle of frame' state, that caches intermediate commands before submitting them to the GPU.
	/// To exit the state and get back the ownership of the [Device](struct.Device.html) call [finish_frame method](struct.DrawingDevice.html#method.finish_frame.html).
	#[inline]
	pub fn start_frame(
		self,
		when: Option<Box<dyn GpuFuture>>,
		final_pass: &impl GraphicalPass,
		clear_value: Vec<vulkano::format::ClearValue>
	) -> Result<DrawingDevice, (Self, vulkano::swapchain::AcquireError)> {
		let (image_index, image_acquire_time) = match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
			Ok(result) => result,
			Err(err) => return Err((self, err)),
		};

		let time: Box<dyn GpuFuture> = match when {
			Some(mut time) => {
				time.cleanup_finished();
				Box::new(time.join(image_acquire_time))
			},
			None => Box::new(vulkano::sync::now(self.device.clone()).join(image_acquire_time)),
		};

		let commands = {
			AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.graphics_queue.family()).unwrap()
				.begin_render_pass(final_pass.framebuffers()[image_index].clone(), false, clear_value).unwrap()
		};

		let draw_device = DrawingDevice {
			device: self,
			time,
			commands,
			image_index
		};
		Ok(draw_device)
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

impl DrawingDevice {
	/// Draw some data.
	/// 
	/// The exact result depends highly on the [GraphicalPass](traits.GraphicalPass.html) in question.
	/// Push-constants should correspond to the ones in the shader used for creating the [GraphicalPass](traits.GraphicalPass.html).
	#[inline]
	pub fn draw<PC>(
		self,
		pass: &impl GraphicalPass,
		vertex_buffer: Vec<Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>>,
		push_constants: PC
	) -> Self {
		let commands = self.commands.draw(pass.pipeline(), pass.dynamic_state(), vertex_buffer, (), push_constants).unwrap();
		DrawingDevice { commands, .. self }
	}

	/// Finish drawing the frame and flush the commands to the GPU.
	/// Note that it does not block execution until the frame is done, rather providing a GpuFuture for when the frame will have been drawn.
	#[inline]
	pub fn finish_frame(self) -> (Device, Result<Box<dyn GpuFuture>, FrameFinishError>) {
		let commands = self.commands.end_render_pass().unwrap().build().unwrap();
		let after_execute = match self.time.then_execute(self.device.graphics_queue.clone(), commands) {
			Ok(future) => future,
			Err(err) => return (self.device, Err(FrameFinishError::Commands(err))),
		};

		let after_flush = after_execute.then_swapchain_present(self.device.graphics_queue.clone(), self.device.swapchain.clone(), self.image_index)
			.then_signal_fence_and_flush();
		
		match after_flush {
			Ok(future) => (self.device, Ok(Box::new(future))),
			Err(err) => (self.device, Err(FrameFinishError::Flush(err))),
		}
	}
}


impl From<vulkano::device::DeviceCreationError> for DeviceCreationError {
	fn from(err: vulkano::device::DeviceCreationError) -> DeviceCreationError { DeviceCreationError::Logical(err) }
}
impl From<vulkano::swapchain::SurfaceCreationError> for DeviceCreationError {
	fn from(err: vulkano::swapchain::SurfaceCreationError) -> DeviceCreationError { DeviceCreationError::Surface(err) }
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
	graphics_queue: &Arc<DeviceQueue>
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
		vulkano::swapchain::PresentMode::Fifo,
		true,
		None);
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