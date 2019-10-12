use std::sync::Arc;

use vulkano::instance::{PhysicalDevice, Version};
use vulkano::device::DeviceExtensions;


use super::context::Context;

pub const REQUIRED_VULKAN_VERSION: Version = Version { major: 1, minor: 0, patch: 0 };

// A graphical device responsible for using hardware acceleration. 
pub struct Device {
	logical: Arc<vulkano::device::Device>,

	graphics_queue: Arc<vulkano::device::Queue>,
	transfer_queue: Arc<vulkano::device::Queue>,
	compute_queue: Arc<vulkano::device::Queue>,
}

#[derive(Debug)]
pub enum DeviceCreationError {
	NoPhysicalDevicesFound, // There were no physical devices to chose from
	NoCompatiblePhysicalDeviceFound, // Some physical devices were found but were not applicable for gaclen
	Logical(vulkano::device::DeviceCreationError), // Error during the creation of logical device
}

impl Device {
	pub fn new(context: &Context) -> Result<Device, DeviceCreationError> {
		let physical = select_physical_device(context)?;

		let device_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
		let queues = select_queue_families(&physical);
		let (logical, mut queues) = vulkano::device::Device::new(physical, physical.supported_features(), &device_extensions, queues.iter().cloned())?;
		let graphics_queue = queues.next().unwrap();
		let transfer_queue = queues.next().unwrap();
		let compute_queue = queues.next().unwrap();

		let device = Device {
			logical,
			graphics_queue,
			transfer_queue,
			compute_queue,
		};
		Ok(device)
	}
}

impl From<vulkano::device::DeviceCreationError> for DeviceCreationError {

	fn from(error: vulkano::device::DeviceCreationError) -> DeviceCreationError { DeviceCreationError::Logical(error) }
}

fn select_physical_device(context: &Context) -> Result<PhysicalDevice, DeviceCreationError> {
	let mut devices = PhysicalDevice::enumerate(&context.instance);
	let mut device = match devices.next() {
		Some(device) => device,
		None => return Err(DeviceCreationError::NoPhysicalDevicesFound),
	};

	for other in devices { device = choose_better_device(device, other); };
	
	match physical_device_is_compatible(&device) {
		true => Ok(device),
		false => Err(DeviceCreationError::NoCompatiblePhysicalDeviceFound),
	}
}

fn choose_better_device<'a>(first: PhysicalDevice<'a>, second: PhysicalDevice<'a>) -> PhysicalDevice<'a> {
	if !physical_device_is_compatible(&second) { return first; };

	// TODO: actual comparison
	first
}

fn physical_device_is_compatible<'a>(device: &PhysicalDevice<'a>) -> bool {
	if cfg!(debug_assertions) {
		println!("Validating device:");
		print_physical_device_details(device, "  ", "    ");
	}

	if device.api_version() < REQUIRED_VULKAN_VERSION { return false; }

	let mut supports_graphics = false;
	let mut supports_compute = false;

	for family in device.queue_families() {
		supports_graphics = supports_graphics || (family.queues_count() > 0 && family.supports_graphics());
		supports_compute = supports_compute || (family.queues_count() > 0 && family.supports_compute());
	};

	supports_compute && supports_graphics
}

fn print_physical_device_details<'a>(device: &PhysicalDevice<'a>, prefix: &str, queue_family_prefix: &str) {
	println!("{}name: {}", prefix, device.name());
	println!("{}type: {:?}", prefix, device.ty());
	println!("{}api version: {}", prefix, device.api_version());
	println!("{}driver version: {}", prefix, device.driver_version());
	println!("{}memory types count: {}", prefix, device.memory_types().count());
	println!("{}queue families ({}):", prefix, device.queue_families().count());
	for family in device.queue_families() {
		print_queue_family_details(&family, queue_family_prefix);
		println!();
	}
}

fn print_queue_family_details<'a>(family: &vulkano::instance::QueueFamily<'a>, prefix: &str) {
	println!("{}id: {}", prefix, family.id());
	println!("{}count: {}", prefix, family.queues_count());
	println!("{}graphics: {}", prefix, family.supports_graphics());
	println!("{}compute: {}", prefix, family.supports_compute());
	println!("{}transfer: {}", prefix, family.explicitly_supports_transfers());
}

fn select_queue_families<'a>(device: &PhysicalDevice<'a>) -> [(vulkano::instance::QueueFamily<'a>, f32); 3] {
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

	if cfg!(debug_assertions) {
		println!("Selected queue families:");
		println!("Graphics:");
		print_queue_family_details(&graphics, "  ");
		println!("Transfer:");
		print_queue_family_details(&transfer, "  ");
		println!("Compute:");
		print_queue_family_details(&compute, "  ");
	}

	[
		(graphics, 1.0),
		(transfer, 0.5),
		(compute, 0.25),
	]
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