//! Device is a logical handle to a hardware-backed instance of a graphical device (typically a GPU).

use super::context::Context;

use std::sync::Arc;

use vulkano::device::{Device as LogicalDevice, DeviceExtensions, Queue as DeviceQueue};
use vulkano::instance::PhysicalDevice;
use vulkano::sync::{GpuFuture};

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

	/// Get the underlying vulkano logical device.
	/// 
	/// The result can be useful for creating simple resources that don't require much usage of gaclen's functionality.
	pub fn logical_device(&self) -> Arc<LogicalDevice> { self.device.clone() }
}

#[cfg(feature = "expose-underlying-vulkano")]
impl Device {
	/// Get the [vulkano device queue](struct.DeviceQueue.html) used for graphical operations.
	#[inline(always)]
	pub fn graphics_queue(&self) -> &Arc<DeviceQueue> { self.graphics_queue }
	/// Get the [vulkano device queue](struct.DeviceQueue.html) used for transfer operations.
	#[inline(always)]
	pub fn transfer_queue(&self) -> &Arc<DeviceQueue> { self.transfer_queue }
	/// Get the [vulkano device queue](struct.DeviceQueue.html) used for compute operations.
	#[inline(always)]
	pub fn compute_queue(&self) -> &Arc<DeviceQueue> { self.compute_queue }
	/// Get the [vulkano swapchian](struct.Swapchain.html) used for presenting images on the screen.
	#[inline(always)]
	pub fn swapchain(&self) -> &Arc<Swapchain<Arc<Window>>> { self.swapchain }
	/// Get the [vulkano swapchain images](struct.SwapchainImage.html) that are presented on the screen.
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

	// prefer exclusively transfer operations
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
