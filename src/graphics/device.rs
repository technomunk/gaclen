use crate::window::Window;
use super::context::Context;
use super::ResizeError;
use super::pipeline::Pass;

use std::sync::Arc;

use vulkano::instance::PhysicalDevice;
use vulkano::device::{Device as LogicalDevice, DeviceExtensions, Queue as DeviceQueue};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreationError, PresentFuture};
use vulkano::image::SwapchainImage;
use vulkano::sync::{GpuFuture, FenceSignalFuture, FlushError};
use vulkano::command_buffer::{CommandBuffer, CommandBufferExecError, CommandBufferExecFuture};


type ImageFormat = (vulkano::format::Format, vulkano::swapchain::ColorSpace);

// A graphical device responsible for using hardware acceleration.
pub struct Device {
	pub(super) device: Arc<LogicalDevice>,

	pub(super) graphics_queue: Arc<DeviceQueue>,
	pub(super) transfer_queue: Arc<DeviceQueue>,
	pub(super) compute_queue: Arc<DeviceQueue>,

	pub(super) swapchain: Arc<Swapchain<Arc<Window>>>,
	pub(super) swapchain_images: Vec<Arc<SwapchainImage<Arc<Window>>>>,
}


#[derive(Debug)]
pub enum DeviceCreationError {
	NoPhysicalDevicesFound, // There were no physical devices to chose from
	NoCompatiblePhysicalDeviceFound, // Some physical devices were found but were not applicable for gaclen
	Logical(vulkano::device::DeviceCreationError), // Error during the creation of logical device
	Surface(vulkano::swapchain::SurfaceCreationError), // Error during the creation of the draw surface
	SurfaceCapabilities(vulkano::swapchain::CapabilitiesError), // Error querying draw surface capabilities
	Swapchain(SwapchainCreationError), // Error during the creation of the swapchain
	NoCompatibleFormatFound, // No compatible format for window draw surface was found
	UnsizedWindow, // Window has no inner size
}

impl Device {
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

	pub fn window_resized(&mut self, window: &Window) -> Result<(), ResizeError> {
		let dimensions: (u32, u32) = match window.get_inner_size() {
			Some(size) => size.into(),
			None => return Err(ResizeError::UnsizedWindow),
		};

		let (swapchain, images) = self.swapchain.recreate_with_dimension([dimensions.0, dimensions.1])?;
		self.swapchain = swapchain;
		self.swapchain_images = images;
		Ok(())
	}

	#[inline]
	pub fn get_frame_end(&self) -> impl GpuFuture {
		vulkano::sync::now(self.device.clone())
	}

	#[inline]
	pub fn acquire_next_image(&self) -> Result<(usize, vulkano::swapchain::SwapchainAcquireFuture<Arc<Window>>), vulkano::swapchain::AcquireError> {
		vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None)
	}

	// TODO: cleanup this
	#[inline]
	pub fn build_draw_command_buffer<PC>(
		&self,
		pass: &Pass,
		vertex_buffer: &Vec<Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>>,
		clear_color: [f32; 4],
		push_constants: PC
	) -> Result<(vulkano::command_buffer::AutoCommandBuffer, vulkano::swapchain::SwapchainAcquireFuture<Arc<Window>>, usize), ()> {
		let (image_num, acquire_future) = self.acquire_next_image().unwrap();
		let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.graphics_queue.family()).unwrap()
			.begin_render_pass(pass.framebuffers[image_num].clone(), false, vec!(clear_color.into())).unwrap()
			.draw(pass.graphics_pipeline.clone(), &pass.dynamic_state, vertex_buffer.clone(), (), push_constants).unwrap()
			.end_render_pass().unwrap()
			.build().unwrap();
		Ok((command_buffer, acquire_future, image_num))
	}

	#[inline]
	pub fn execute_after<F, C>(&self, future: F, commands: C)
	-> Result<CommandBufferExecFuture<F, C>, CommandBufferExecError>
	where
		F: GpuFuture,
		C: CommandBuffer + 'static
	{
		future.then_execute(self.graphics_queue.clone(), commands)
	}

	#[inline]
	pub fn present_after<F>(&self, future: F, image_index: usize) -> PresentFuture<F, Arc<Window>>
	where F: GpuFuture
	{
		future.then_swapchain_present(self.graphics_queue.clone(), self.swapchain.clone(), image_index)
	}

	#[inline]
	pub fn flush_after<F>(&self, future: F) -> Result<FenceSignalFuture<F>, FlushError>
	where F: GpuFuture
	{
		future.then_signal_fence_and_flush()
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