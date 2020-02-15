// TODO/rel: clean up this explanation

//! The device uses GPU to execute rendering and computing commands.
//! 
//! In order to get an image using a [Device](struct.Device.html) one needs to:
//! 1. [Create it](struct.Device.html#method.new).
//! 2. [Create a render-pass](struct.GraphicalPassBuilder.html).
//! 3. [Create a data-buffer](struct.Device.html#method.create_cpu_accessible_buffer).
//! 4. [Start a frame](struct.Device.html#method.start_frame).
//! 5. [Start a pass](struct.Frame.html#method.begin_pass).
//! 6. [Draw using a pass](struct.PassInFrame.html#method.draw).
//! 7. [Finish the pass](struct.PassInFrame.html#method.finish_pass).
//! 8. Repeat steps `5-7` as necessary.
//! 9. [Finish the frame](struct.Frame.html#method.finish_frame).
//! 
//! Note that the above has 3 states:
//! - [Device] : normal (default) state, most functionality is available.
//! - [Frame] : the device is in the middle of drawing a frame, only [starting a pass](struct.Frame.html#method.begin_pass) and [finishing the frame](struct.Frame.html#method.finish_frame) are available.
//! - [PassInFrame] : the device is in the middle of drawing a frame with a specific setup ([GraphicalPass]). Only the [drawing](struct.PassInFrame.html#method.draw) and [finishing the pass](struct.PassInFrame.html#method.finish_pass) are available.

use super::context::Context;

use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, ImmutableBuffer};
use vulkano::device::{Device as LogicalDevice, DeviceExtensions, Queue as DeviceQueue};
use vulkano::format::{AcceptsPixels, Format, FormatDesc};
use vulkano::image::{ImageCreationError};
use vulkano::sampler::{Filter, Sampler, SamplerCreationError, SamplerAddressMode, MipmapMode};
use vulkano::instance::PhysicalDevice;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::sync::{GpuFuture};
use vulkano::memory::DeviceMemoryAllocError;

pub use vulkano::swapchain::PresentMode;

/// A device responsible for hardware-accelerated computations.
/// 
/// It is responsible for recording, submitting and synchronizing commands and data to the GPU.
/// The device structure contains some state information for synchronization purposes.
pub struct Device {
	pub(super) device: Arc<LogicalDevice>,

	pub(super) graphics_queue: Arc<DeviceQueue>,
	pub(super) transfer_queue: Arc<DeviceQueue>,
	pub(super) compute_queue: Arc<DeviceQueue>,

	pub(super) before_frame: Option<Box<dyn GpuFuture>>,
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
}

/// Error during [Framebuffer] creation.
pub enum FramebufferCreationError {

}

// General
impl Device {
	/// Create a new device using provided driver context.
	pub fn new(
		context: &Context,
	) -> Result<Device, DeviceCreationError>
	{
		let physical = select_physical_device(context)?;

		let device_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
		let queues = select_queue_families(&physical);
		let (logical, queues) = LogicalDevice::new(physical, physical.supported_features(), &device_extensions, queues.iter().cloned())?;
		let [graphics_queue, transfer_queue, compute_queue] = unpack_queues(queues.collect());

		Ok(Device {
			device: logical,
			graphics_queue,
			transfer_queue,
			compute_queue,
			before_frame: None,
		})
	}

	/// Get the PhysicalDevice selected when this Device was created.
	pub fn physical_device(&self) -> PhysicalDevice {
		self.device.physical_device()
	}

	/// Get the underlying logical device (useful for supplying to own shaders).
	pub fn logical_device(&self) -> Arc<LogicalDevice> { self.device.clone() }
}

// Buffers
impl Device {
	/// Create a basic buffer for data for processing on the GPU.
	/// 
	/// These are useful for quick prototyping, as these are typically placed in shared memory (system based RAM, accessed by GPU through the bus).
	pub fn create_cpu_accessible_buffer<T: 'static>(&self, data_iterator: impl ExactSizeIterator<Item = T>) -> Result<Arc<CpuAccessibleBuffer<[T]>>, DeviceMemoryAllocError> {
		CpuAccessibleBuffer::from_iter(self.logical_device(), BufferUsage::all(), data_iterator)
	}

	/// Create a pool of small CPU-accessible buffers.
	/// 
	/// These are particularly useful for regularly changing data, such as object uniforms.
	pub fn create_cpu_buffer_pool<T: 'static>(&self, usage: BufferUsage) -> CpuBufferPool<T> {
		CpuBufferPool::<T>::new(self.logical_device(), usage)
	}

	/// Create a device-local immutable buffer from some data.
	/// 
	/// Builds an intermediate memory-mapped buffer, writes data to it, builds a copy (upload) command buffer and executes it.
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to submit the copy command buffer.
	pub fn create_immutable_buffer_from_data<T>(&self, data: T, usage: BufferUsage) -> Result<Arc<ImmutableBuffer<T>>, DeviceMemoryAllocError>
	where
		T : Send + Sync + Sized + 'static,
	{
		let (buffer, future) = ImmutableBuffer::from_data(data, usage, self.transfer_queue.clone())?;

		// TODO: handle synchronization between separate queues in a performant way
		future.flush().unwrap();

		Ok(buffer)
	}

	/// Create a device-local immutable buffer from some data iterator.
	/// 
	/// Builds an intermediate memory-mapped buffer, writes data to it, builds a copy (upload) command buffer and executes it.
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to submit the copy command buffer.
	pub fn create_immutable_buffer_from_iter<T>(&self, data_iterator: impl ExactSizeIterator<Item = T>, usage: BufferUsage) -> Result<Arc<ImmutableBuffer<[T]>>, DeviceMemoryAllocError>
	where
		T : Send + Sync + Sized + 'static,
	{
		let (buffer, future) = ImmutableBuffer::from_iter(data_iterator, usage, self.transfer_queue.clone())?;

		// TODO: handle synchronization between separate queues in a performant way
		future.flush().unwrap();

		Ok(buffer)
	}

	// TODO: add more ways to create buffers
}

// Images
impl Device {
	/// Create an [ImmutableImage] from a data iterator.
	/// 
	/// Builds an intermediate memory-mapped buffer, writes data to it, builds a copy (upload) command buffer and executes it.
	/// 
	/// # Panic.
	/// 
	/// - Panics if fails to submit the copy command buffer.
	pub fn create_immutable_image_from_iter<P, I, F>(&self, data_iterator: I, dimensions: Dimensions, format: F)
	-> Result<Arc<ImmutableImage<F>>, ImageCreationError>
	where
		P : Send + Sync + Clone + 'static,
		F : FormatDesc + AcceptsPixels<P> + Send + Sync + 'static,
		I : ExactSizeIterator<Item = P>,
		Format: AcceptsPixels<P>,
	{
		let (image, future) = ImmutableImage::from_iter(data_iterator, dimensions, format, self.transfer_queue.clone())?;

		// TODO: handle synchronization between separate queues in a performant way
		future.flush().unwrap();

		// let time: Box<dyn GpuFuture> = match self.before_frame.take() {
		// 	Some(time) => Box::new(time.join(future)),
		// 	None => Box::new(future),
		// };
		// self.before_frame = Some(time);

		Ok(image)
	}

	/// Creates a new [Sampler] (image viewer) with the given behavior.
    ///
    /// `magnifying_filter` and `minifying_filter` define how the implementation should sample from the image
    /// when it is respectively larger and smaller than the original.
    ///
    /// `mipmap_mode` defines how the implementation should choose which mipmap to use.
    ///
    /// `address_u`, `address_v` and `address_w` define how the implementation should behave when
    /// sampling outside of the texture coordinates range `[0.0, 1.0]`.
    ///
    /// `mip_lod_bias` is a value to add to .
    ///
    /// `max_anisotropy` must be greater than or equal to 1.0. If greater than 1.0, the
    /// implementation will use anisotropic filtering. Using a value greater than 1.0 requires
    /// the `sampler_anisotropy` feature to be enabled when creating the device.
    ///
    /// `min_lod` and `max_lod` are respectively the minimum and maximum mipmap level to use.
    /// `max_lod` must always be greater than or equal to `min_lod`.
    ///
    /// # Panic
    ///
    /// - Panics if multiple `ClampToBorder` values are passed and the border color is different.
    /// - Panics if `max_anisotropy < 1.0`.
    /// - Panics if `min_lod > max_lod`.
	pub fn create_sampler(
		&self,
		magnifying_filter: Filter,
		minifying_filter: Filter,
		mipmap_mode: MipmapMode,
		address_u: SamplerAddressMode,
		address_v: SamplerAddressMode,
		address_w: SamplerAddressMode,
		mip_lod_bias: f32,
		max_anisotropy: f32,
		min_lod: f32,
		max_lod: f32
	) -> Result<Arc<Sampler>, SamplerCreationError> {
		Sampler::new(
			self.device.clone(),
			magnifying_filter,
			minifying_filter,
			mipmap_mode,
			address_u,
			address_v,
			address_w,
			mip_lod_bias,
			max_anisotropy,
			min_lod,
			max_lod
		)
	}

	/// Shortcut for creating a simple sampler with default settings that is useful for prototyping.
	pub fn create_simple_linear_repeat_sampler(&self) -> Result<Arc<Sampler>, SamplerCreationError>
	{
		self.create_sampler(
			Filter::Linear,
			Filter::Linear,
			MipmapMode::Linear,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0,
			1.0,
			0.0,
			1_000.0
		)
	}
}

// Member exposure
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

impl From<vulkano::device::DeviceCreationError> for DeviceCreationError {
	fn from(err: vulkano::device::DeviceCreationError) -> DeviceCreationError { DeviceCreationError::Logical(err) }
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
